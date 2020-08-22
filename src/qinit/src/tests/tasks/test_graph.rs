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