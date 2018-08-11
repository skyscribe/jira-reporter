extern crate serde;
extern crate serde_json;

#[derive(Serialize, Deserialize)]
struct Person{
    name: String,
    age: u8,
    phones: Vec<String>,
}

#[allow(dead_code)]
pub fn test_typed() {
    let data = r#"{
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 12345",
                "+44 23456"
            ]
        }"#;
    
    let p:Person = serde_json::from_str(data).unwrap();
    assert_eq!(p.name, "John Doe");
    assert_eq!(p.age, 43);
    assert_eq!(p.phones.len(), 2);
    assert_eq!(p.phones[0], "+44 12345");
    assert_eq!(p.phones[1], "+44 23456");
}