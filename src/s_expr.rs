use lexpr::{Cons, Value};

pub type Node = Cons;

pub fn node_get_attr<'a>(node: &'a Node, name: &str) -> &'a Value {
    node.iter()
        .skip(1)
        .filter(|&x| x.car().as_keyword() == Some(name))
        .nth(0)
        .unwrap()
        .cdr()
        .as_cons()
        .unwrap()
        .car()
}

pub fn node_add_attr(node: Node, name: &str, value: Value) -> Node {
    let (car, cdr) = node.into_pair();
    Cons::new(
        car,
        Value::cons(Value::keyword(name), Value::cons(value, cdr)),
    )
}

pub fn node_get_field<'a>(node: &'a Node, i: usize) -> &'a Value {
    node_get_fields(node)[i]
}

pub fn node_get_fields<'a>(node: &'a Node) -> Vec<&'a Value> {
    let mut res = vec![];
    let mut skip = false;
    for elem in node.list_iter().skip(1) {
        if skip {
            skip = false;
            continue;
        }

        if elem.is_keyword() {
            skip = true;
            continue;
        }

        res.push(elem);
    }
    res
}
