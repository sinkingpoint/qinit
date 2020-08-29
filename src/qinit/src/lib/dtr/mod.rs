use std::cmp::Eq;
use std::fmt;
use std::string::ToString;
use std::collections::HashMap;

type IndexType = usize;

struct Node<T: Eq> {
    data: T,
    edges: Vec<IndexType>
}

impl<T: Eq> PartialEq<Node<T>> for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        return &self.data == &other.data;
    }
}

impl<T: Eq> PartialEq<T> for Node<T> {
    fn eq(&self, other: &T) -> bool {
        return &self.data == other;
    }
}

impl<T: Eq> Node<T> {
    fn new(data: T) -> Node<T> {
        return Node {
            data: data,
            edges: Vec::new()
        }
    }
}

struct Edge<T> {
    data: T,
    from: IndexType,
    to: IndexType
}

/// Graph represents a directed graph with nodes and edges both carrying arbitrary data
/// Internally we use the adjancency list pattern to handle these relationships
pub struct Graph<N: Eq, E> { 
    /// The list of nodes in the graph
    nodes: Vec<Node<N>>,

    /// The list of edges in the graph
    edges: Vec<Edge<E>>
}

impl<N: Eq, E> Graph<N, E> {
    pub fn new() -> Graph<N, E> {
        return Graph {
            nodes: Vec::new(),
            edges: Vec::new()
        }
    }

    pub fn iter_decendents(&self, node: &N) -> Option<impl Iterator<Item = &N> + '_> {
        let node_index = match self.get_index_for_node(node) {
            Some(idx) => idx,
            None => {
                return None;
            }
        };

        return Some(self.nodes[node_index].edges.iter().filter(move |&&edge| self.edges[edge].from == node_index).map(move |&edge| &self.nodes[self.edges[edge].to].data));
    }

    pub fn iter_leaves(&self) -> impl Iterator<Item = &N> + '_ {
        return self.nodes.iter().enumerate().filter(move |(node_index, node)| node.edges.iter().filter(|&edge_index| self.edges[*edge_index].from == *node_index).collect::<Vec<&IndexType>>().len() == 0).map(|(_, node)| &node.data);
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &N> + '_ {
        return self.nodes.iter().map(|node| &node.data);
    }

    pub fn extend(&mut self, other: Graph<N, E>) {
        let mut node_index_translator = HashMap::new();
        for (i, node) in other.nodes.into_iter().enumerate() {
            let new_index = self.add_node(node.data);
            node_index_translator.insert(i, new_index);
        }

        for edge in other.edges.into_iter() {
            self.add_edge_by_index(*node_index_translator.get(&edge.from).unwrap(), *node_index_translator.get(&edge.to).unwrap(), edge.data);
        }
    }

    /// add_node adds the given data value as an unconnected node in the graph
    /// returning the new index of this data which can then be used to add/remove edges
    pub fn add_node(&mut self, node: N) -> IndexType{
        if let Some(index) = self.nodes.iter().position(|n| n.data == node) {
            return index;
        }

        self.nodes.push(Node::new(node));
        return self.nodes.len() - 1;
    }

    pub fn find_node_index<P>(&self, pred: P) -> Option<IndexType> where P: FnMut(&N) -> bool {
        return self.nodes.iter().map(|node| &node.data).position(pred);
    }

    pub fn get_node_by_index(&self, idx: IndexType) -> Option<&N> {
        if idx >= self.nodes.len() {
            return None;
        }

        return Some(&self.nodes[idx].data);
    }

    /// get_index_for_node returns the first index in the node vec that has
    /// the given data value
    #[inline(always)]
    pub fn get_index_for_node(&self, data: &N) -> Option<IndexType> {
        return self.nodes.iter().position(|n| n == data);
    }

    /// add_edge_by_index adds an index to the graph, between the two nodes identified by the given 
    /// indices. Exits silenty if either of the given indices are invalid
    pub fn add_edge_by_index(&mut self, from_index: IndexType, to_index: IndexType, edge: E) -> Option<IndexType> {
        if from_index >= self.nodes.len() || to_index >= self.nodes.len() {
            return None;
        }

        let edge_index = self.edges.len();
        self.edges.push(Edge {
            data: edge,
            from: from_index,
            to: to_index
        });

        self.nodes[from_index].edges.push(edge_index);
        self.nodes[to_index].edges.push(edge_index);
        return Some(self.edges.len() - 1);
    }

    /// add_edge adds the given edge value as an edge between the given two node values. If two nodes
    /// exist with the same value, which one will be linked to the edge is undefined
    pub fn add_edge(&mut self, from: &N, to: &N, edge: E) -> Option<IndexType> {
        let from_index = match self.get_index_for_node(from) {
            Some(idx) => idx,
            None => {
                return None;
            }
        };

        let to_index = match self.get_index_for_node(to) {
            Some(idx) => idx,
            None => {
                return None;
            }
        };

        return self.add_edge_by_index(from_index, to_index, edge);
    }

    /// remove_edge_by_index removes the edge identified by the given index from this graph
    /// This action invalidates any previously issued edge indices
    pub fn remove_edge_by_index(&mut self, edge_index: IndexType) {
        if edge_index >= self.edges.len() {
            return;
        }

        let edge = &self.edges[edge_index];

        // Remove the references on either side of this edge
        self.nodes[edge.to].edges.retain(|&idx| idx != edge_index);
        self.nodes[edge.from].edges.retain(|&idx| idx != edge_index);

        // swap_remove this edge, so the last edge takes its place
        self.edges.swap_remove(edge_index);
        let old_index = self.edges.len();

        if old_index == 0 || old_index == edge_index {
            // We just removed the last edge, or the last added edge, so nothing to update
            return;
        }

        // edges[edge_index] now contains the value that was at old_index
        // so we need to update the nodes at <to> and <from> with the new index
        let to_index = self.edges[edge_index].to;
        let from_index = self.edges[edge_index].from;

        self.nodes[to_index].edges.retain(|&idx| idx != old_index);
        self.nodes[to_index].edges.push(edge_index);
        self.nodes[from_index].edges.retain(|&idx| idx != old_index);
        self.nodes[from_index].edges.push(edge_index);
    }

    /// remove_edge removes the first edge found between two nodes represented by the given data `from` and `to`
    /// This invalidates any previously returned edge indices
    pub fn remove_edge(&mut self, from: &N, to: &N) {
        let edge_index = match self.edges.iter().position(|edge| &self.nodes[edge.to] == to && &self.nodes[edge.from] == from) {
            Some(idx) => idx,
            None => {
                return;
            }
        };

        self.remove_edge_by_index(edge_index);
    }

    /// remove_node_by_index removes the node identified by the given index
    /// This invalidates all previously returned node indices
    /// It operates in O(n + m) time, where n is the number of edges connected to this node
    /// and m is the number of edges attached to the last node added
    pub fn remove_node_by_index(&mut self, index: IndexType) {
        if index >= self.nodes.len() {
            return;
        }

        // For all the edges attached to this node, kill them
        let edges_to_remove: Vec<IndexType> = self.nodes[index].edges.iter().map(|x| *x).collect();
        for edge_index in edges_to_remove.iter() {
            self.remove_edge_by_index(*edge_index);
        }

        // Swap remove - move the last element into the hole, so we only have to update the edges that point to one node
        self.nodes.swap_remove(index);
        let old_index = self.nodes.len();

        if old_index == 0 || index == old_index {
            // We just removed the only node, so no need to update edges (There shouldn't be any)
            return;
        }
        
        // For every edge that points to the last node (that we just moved), update their to and/or from pointers
        for edge_index in self.nodes[index].edges.iter() {
            let mut edge = &mut self.edges[*edge_index];
            if edge.from == old_index {
                edge.from = index;
            }

            if edge.to == old_index {
                edge.to = index;
            }
        }
    }

    /// remove_node removes the first node with the given data value
    /// This invalidates all previously returned node indices
    /// It operates in O(n + m) time, where n is the number of edges connected to this node
    /// and m is the number of edges attached to the last node added
    pub fn remove_node(&mut self, node: &N) {
        let index = match self.get_index_for_node(node) {
            Some(idx) => idx,
            None => {
                return;
            }
        };

        self.remove_node_by_index(index);
    }
}

impl<N: Eq + ToString, E> fmt::Display for Graph<N, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("digraph {\n")?;
        for node in self.nodes.iter() {
            writeln!(f, "\t\"{}\";", node.data.to_string())?;
        }

        for edge in self.edges.iter() {
            writeln!(f, "\t\"{}\" -> \"{}\";", self.nodes[edge.from].data.to_string(), self.nodes[edge.to].data.to_string())?;
        }
        return f.write_str("}");
    }
}
