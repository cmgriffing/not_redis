use not_redis::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new();
    client.start().await;

    // String operations
    client.set("key", "value").await.unwrap();
    let val: String = client.get("key").await.unwrap();
    println!("key = {}", val);

    // Hash operations
    client.hset("user:1", "name", "Alice").await.unwrap();
    client.hset("user:1", "age", "30").await.unwrap();
    let name: String = client.hget("user:1", "name").await.unwrap();
    println!("user:1 name = {}", name);

    // TTL operations
    client.set("temp", "data").await.unwrap();
    client.expire("temp", 60).await.unwrap();
    let ttl: i64 = client.ttl("temp").await.unwrap();
    println!("temp TTL = {} seconds", ttl);

    // List operations
    client.lpush("mylist", "first").await.unwrap();
    client.rpush("mylist", "second").await.unwrap();
    let len: i64 = client.llen("mylist").await.unwrap();
    println!("mylist len = {}", len);

    // Set operations
    client.sadd("myset", "member1").await.unwrap();
    let members: Vec<String> = client.smembers("myset").await.unwrap();
    println!("myset members = {:?}", members);

    println!("All tests passed!");
}
