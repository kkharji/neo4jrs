# neo4jrs [![Crates.io][crates-badge]][crates-url]

[ci-url]: https://github.com/tami5/neo4jrs
[crates-badge]: https://img.shields.io/crates/v/neo4jrs.svg?style=shield
[crates-url]: https://crates.io/crates/neo4jrs
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=shield
[docs-url]: https://docs.rs/neo4jrs

neo4j.rs is a Neo4j rust driver implemented using [bolt specification](https://7687.org/bolt/bolt-protocol-message-specification-4.html#version-41)

This driver is compatible with neo4j 4.x versions

Builds upon and originally a fork of https://github.com/yehohanan7/neo4rs

## API Documentation: [![Docs.rs][docs-badge]][docs-url]

## Example

```rust
    // concurrent queries
    let uri = "127.0.0.1:7687";
    let user = "neo4j";
    let pass = "neo";
    let graph = Arc::new(Graph::new(&uri, user, pass).await.unwrap());
    for _ in 1..=42 {
        let graph = graph.clone();
        tokio::spawn(async move {
            let mut result = graph.execute(
	       query("MATCH (p:Person {name: $name}) RETURN p").param("name", "Mark")
	    ).await.unwrap();
            while let Ok(Some(row)) = result.next().await {
        	let node: Node = row.get("p").unwrap();
        	let name: String = node.get("name").unwrap();
                println!("{}", name);
            }
        });
    }
    //Transactions
    let mut txn = graph.start_txn().await.unwrap();
    txn.run_queries(vec![
        query("CREATE (p:Person {name: 'mark'})"),
        query("CREATE (p:Person {name: 'jake'})"),
        query("CREATE (p:Person {name: 'luke'})"),
    ])
    .await
    .unwrap();
    txn.commit().await.unwrap(); //or txn.rollback().await.unwrap();
```


## License

Neo4jrs is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
