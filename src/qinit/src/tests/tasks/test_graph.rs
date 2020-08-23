extern crate accountant;

use accountant::dtr::Graph;
#[test]
fn test_graph_adds_nodes() {
    let mut graph = Graph::new();
    
    graph.add_node(1);
    graph.add_node(2);
    graph.add_node(3);

    graph.add_edge(&1, &2, 10);
    graph.add_edge(&3, &1, 20);
    graph.add_edge(&1, &3, 30);
    graph.add_edge(&2, &3, 40);

    let iter = graph.iter_decendents(&1);
    assert!(iter.is_some());
    let mut iter = iter.unwrap();
    let val = iter.next();
    assert!(val.is_some());
    assert_eq!(val.unwrap(), (&10, &2));

    let val = iter.next();
    assert!(val.is_some());
    assert_eq!(val.unwrap(), (&30, &3));

    assert!(iter.next().is_none());
}

#[test]
fn test_graph_removes_nodes() {
    let mut graph = Graph::<u32, u32>::new();
    let idx = graph.add_node(1);
    graph.remove_node_by_index(idx);

    assert!(graph.iter_decendents(&1).is_none());

    let idx = graph.add_node(1);
    graph.add_node(2);
    graph.remove_node_by_index(idx);
    assert!(graph.iter_decendents(&1).is_none());
    let iter = graph.iter_decendents(&2);
    assert!(iter.is_some());
    assert!(iter.unwrap().next().is_none());
}

#[test]
fn test_graph_removes_edges() {
    let mut graph = Graph::<u32, u32>::new();
    graph.add_node(1);
    graph.add_node(2);
    let idx = graph.add_edge(&1, &2, 10);
    let iter = graph.iter_decendents(&1);
    assert!(iter.is_some());
    let mut iter = iter.unwrap();
    let val = iter.next();
    assert!(val.is_some());
    assert_eq!(val.unwrap(), (&10, &2));

    assert!(idx.is_some());
    graph.remove_edge_by_index(idx.unwrap());

    let iter = graph.iter_decendents(&1);
    assert!(iter.is_some());
    assert!(iter.unwrap().next().is_none());
}