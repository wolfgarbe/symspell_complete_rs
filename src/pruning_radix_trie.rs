use itertools::Itertools;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::cmp::{max, min};
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use compact_str::CompactString;

#[derive(Copy, Clone)]
struct NodeId(usize);
type NodeKey = CompactString;
struct Node {
    children: Option<Vec<(NodeKey, NodeId)>>,
    weight: Option<usize>,
    child_max_weight: Option<usize>,
}

/// PruningRadixTrie for autocomplete.
pub struct PruningRadixTrie {
    nodes: Vec<Node>,
    term_count: usize,
}

/// Suggested completion for a given substring.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Completion<'a> {
    /// The suggested correctly spelled word.
    pub term: NodeKey,
    /// Frequency of suggestion in the dictionary (a measure of how common the suggestion is).
    pub count: &'a usize,
}

impl<'a> Completion<'a> {
    /// Creates a new PruningRadixTrie instance.
    pub fn new(term: NodeKey, count: &'a usize) -> Self {
        Self { term, count }
    }
}

//?????
struct NodeMatchContext<'a> {
    node_id: NodeId,
    node_index: usize,
    common: usize,
    key: &'a str,
}

//?????
enum NodeMatch<'a> {
    NoMatch,
    Equal(NodeMatchContext<'a>),
    IsShorter(NodeMatchContext<'a>),
    IsLonger(NodeMatchContext<'a>),
    CommonSubstring(NodeMatchContext<'a>),
}

impl<'a> Ord for Completion<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Order by count descending (tie-break by completion for stability)
        other.count.cmp(self.count).then(self.term.cmp(&other.term))
    }
}
impl<'a> PartialOrd for Completion<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for Completion<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl<'a> Eq for Completion<'a> {}

impl Default for PruningRadixTrie {
    fn default() -> Self {
        PruningRadixTrie::new()
    }
}

impl PruningRadixTrie {
    fn match_children(&self, node_id: NodeId, term: &str) -> NodeMatch<'_> {
        if let Some(children) = &self.nodes[node_id.0].children {
            for (index, (key, id)) in children.iter().enumerate() {
                let mut common = 0;
                for i in 0..min(term.len(), key.len()) {
                    if term.as_bytes().get(i).unwrap() == key.as_bytes().get(i).unwrap() {
                        common = i + 1;
                    } else {
                        break;
                    }
                }
                while !term.is_char_boundary(common) {
                    common -= 1;
                }
                if common > 0 {
                    let context = NodeMatchContext {
                        node_id: *id,
                        node_index: index,
                        common,
                        key,
                    };
                    return match (common == term.len(), common == key.len()) {
                        (true, true) => NodeMatch::Equal(context),
                        (true, _) => NodeMatch::IsShorter(context),
                        (_, true) => NodeMatch::IsLonger(context),
                        (_, _) => NodeMatch::CommonSubstring(context),
                    };
                }
            }
        }

        NodeMatch::NoMatch
    }

    fn make_node(
        &mut self,
        children: Option<Vec<(NodeKey, NodeId)>>,
        weight: Option<usize>,
        child_max_weight: Option<usize>,
    ) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            children,
            weight,
            child_max_weight,
        });
        node_id
    }

    fn append_child<S: Into<NodeKey>>(&mut self, parent_id: NodeId, term: S, child_id: NodeId) {
        let child_node = &self.nodes[child_id.0];
        let insert_index =
            self.get_insert_index(parent_id, child_node.weight, child_node.child_max_weight);

        let parent_node = &mut self.nodes[parent_id.0];
        if let Some(children) = &mut parent_node.children {
            children.insert(insert_index, (term.into(), child_id));
        } else {
            parent_node.children = Some(vec![(term.into(), child_id)]);
        }
    }

    fn find_all_child_terms<'a>(
        &'a self,
        node: &'a Node,
        prefix: &str,
        matched_prefix: &mut NodeKey,
        top_k: usize,
        results: &mut BinaryHeap<Completion<'a>>,
    ) {
        if let Some(children) = &node.children {
            if results.len() == top_k && node.child_max_weight <= results.peek().map(|r| *r.count) {
                return;
            }
            for (term, child_id) in children {
                let child = &self.nodes[child_id.0];

                if results.len() == top_k
                    && child.weight <= results.peek().map(|r| *r.count)
                    && child.child_max_weight <= results.peek().map(|r| *r.count)
                {
                    if prefix.is_empty() {
                        continue;
                    } else {
                        break;
                    }
                }

                if prefix.is_empty() || term.starts_with(prefix) {
                    if child.weight.is_some() || node.children.is_some() {
                        matched_prefix.push_str(term);

                        if let Some(weight) = child.weight.as_ref() {
                            results.push(Completion {
                                term: matched_prefix.as_str().into(),
                                count: weight,
                            });
                            if results.len() > top_k {
                                results.pop();
                            }
                        }
                        self.find_all_child_terms(child, "", matched_prefix, top_k, results);
                        matched_prefix.truncate(matched_prefix.len() - term.len());
                    }

                    if !prefix.is_empty() {
                        break;
                    }
                } else if prefix.starts_with(term.as_str()) {
                    matched_prefix.push_str(term);
                    self.find_all_child_terms(
                        child,
                        &prefix[term.len()..],
                        matched_prefix,
                        top_k,
                        // predicate,
                        results,
                    );
                    matched_prefix.truncate(matched_prefix.len() - term.len());
                }
            }
        }
    }

    fn get_insert_index(
        &self,
        node_id: NodeId,
        weight: Option<usize>,
        child_max_weight: Option<usize>,
    ) -> usize {
        if let Some(children) = &self.nodes[node_id.0].children {
            let result = children.binary_search_by(|(_, child_id)| {
                match child_max_weight.cmp(&self.nodes[child_id.0].child_max_weight) {
                    Equal => weight.cmp(&self.nodes[child_id.0].weight),
                    Less => Less,
                    Greater => Greater,
                }
            });
            match result {
                Ok(index) => index,
                Err(index) => index,
            }
        } else {
            0
        }
    }

    fn replace_node(&mut self, parent_id: NodeId, node_index: usize, term: &str, child_id: NodeId) {
        self.nodes[parent_id.0]
            .children
            .as_mut()
            .unwrap()
            .remove(node_index);
        self.append_child(parent_id, term.to_owned(), child_id);
    }

    fn update_child_max_weight(
        &mut self,
        parent_id: NodeId,
        node_id: NodeId,
        node_index: usize,
        new_child_max_weight: usize,
    ) {
        let node = &mut self.nodes[node_id.0];
        let new_child_max_weight = Some(new_child_max_weight);
        if node.child_max_weight < new_child_max_weight {
            node.child_max_weight = new_child_max_weight;

            if node_index > 0 {
                let (_, prev_child_id) =
                    self.nodes[parent_id.0].children.as_mut().unwrap()[node_index - 1];
                if node_index > 0
                    || new_child_max_weight > self.nodes[prev_child_id.0].child_max_weight
                {
                    let (term, child_id) = self.nodes[parent_id.0]
                        .children
                        .as_mut()
                        .unwrap()
                        .remove(node_index);
                    self.append_child(parent_id, term, child_id);
                }
            }
        }
    }

    fn add_term(&mut self, curr_id: NodeId, term: &str, weight: usize) -> usize {
        match self.match_children(curr_id, term) {
            NodeMatch::Equal(NodeMatchContext { node_id, .. }) => {
                let node = &mut self.nodes[node_id.0];
                if let Some(node_weight) = node.weight {
                    let new_weight = node_weight + weight;
                    node.weight = Some(new_weight);
                    new_weight
                } else {
                    self.term_count += 1;
                    node.weight = Some(weight);
                    //node.payload = Some(payload);
                    weight
                }
            }

            NodeMatch::IsShorter(NodeMatchContext {
                node_id,
                common,
                node_index,
                key,
            }) => {
                let node = &self.nodes[node_id.0];
                let child_id = self.make_node(
                    Some(vec![(key[common..].into(), node_id)]),
                    Some(weight),
                    max(node.weight, node.child_max_weight),
                );

                self.replace_node(curr_id, node_index, &term[0..common], child_id);
                self.term_count += 1;
                weight
            }

            NodeMatch::IsLonger(NodeMatchContext {
                node_id,
                common,
                node_index,
                ..
            }) => {
                let weight = self.add_term(node_id, &term[common..], weight);
                self.update_child_max_weight(curr_id, node_id, node_index, weight);
                weight
            }

            NodeMatch::CommonSubstring(NodeMatchContext {
                node_id,
                common,
                node_index,
                key,
            }) => {
                let node = &self.nodes[node_id.0];
                let key = key[common..].into();
                let child_max_weight = max(node.child_max_weight, max(node.weight, Some(weight)));
                let new_node_id = self.make_node(None, Some(weight), None);
                let child_id = self.make_node(
                    Some(vec![(key, node_id), (term[common..].into(), new_node_id)]),
                    None,
                    // None,
                    child_max_weight,
                );

                self.replace_node(curr_id, node_index, &term[0..common], child_id);
                self.term_count += 1;
                weight
            }

            NodeMatch::NoMatch => {
                let node_id = self.make_node(None, Some(weight), Default::default());
                self.append_child(curr_id, term.to_owned(), node_id);
                self.term_count += 1;
                weight
            }
        }
    }

    // #####

    /// Creates a new PruningRadixTrie instance.
    pub fn new() -> Self {
        PruningRadixTrie {
            nodes: vec![Node {
                children: None,
                weight: None,
                child_max_weight: None,
            }],
            term_count: 0,
        }
    }

    /// Load completions from a file of string/frequency count pairs.
    ///
    /// # Arguments
    ///
    /// * `path` - The path+filename of the file.
    /// * `term_index` - The column position of the word.
    /// * `count_index` - The column position of the frequency count.
    /// * `separator` - Separator between word and frequency
    pub fn load_completions(
        &mut self,
        path: &Path,
        term_index: usize,
        count_index: usize,
        separator: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let sr = BufReader::new(file);

        for line in sr.lines() {
            let line_str = line?;
            //self.load_dictionary_line(&line_str, term_index, count_index, separator);

            let line_parts: Vec<&str> = line_str.split(separator).collect();
            if line_parts.len() >= 2 {
                self.add_completion(
                    line_parts[term_index],
                    line_parts[count_index].parse::<usize>().unwrap(),
                );
            }
        }

        Ok(())
    }

    /// Get the number of entries in the dictionary.
    pub fn len(&self) -> usize {
        self.term_count
    }

    /// Check if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.term_count == 0
    }

    /// Save completions, ordered descending by count to a file.
    pub fn save_completions(
        &self,
        path: &Path,
        separator: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut results = BinaryHeap::new();
        let mut matched_prefix = NodeKey::default();

        self.find_all_child_terms(
            &self.nodes[0],
            "",
            &mut matched_prefix,
            usize::MAX,
            &mut results,
        );
        for entry in results
            .iter()
            .sorted_unstable_by(|a, b| Ord::cmp(&b.count, &a.count))
        {
            writeln!(writer, "{}{}{}", entry.term, separator, entry.count)?;
        }
        writer.flush()?;

        Ok(())
    }

    /// Add a completion to the dictionary.
    pub fn add_completion(&mut self, term: &str, weight: usize) {
        let weight = self.add_term(NodeId(0), term, weight);
        self.nodes[0].child_max_weight = max(self.nodes[0].child_max_weight, Some(weight));
    }

    /// Lookup completions for a given input prefix.
    /// If top_k is provided, keep only the top-k by count using a min-heap.
    pub fn lookup_completions<'a>(
        &'a self,
        prefix: &str,
        top_k: Option<usize>,
    ) -> Vec<Completion<'a>> {
        let mut results: BinaryHeap<Completion<'a>> = BinaryHeap::new();
        let mut matched_prefix = NodeKey::default();
        self.find_all_child_terms(
            &self.nodes[0],
            prefix,
            &mut matched_prefix,
            top_k.unwrap_or(usize::MAX),
            &mut results,
        );
        // Order by count descending (tie-break by completion for stability)
        results.into_sorted_vec()
    }
}
