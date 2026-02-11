use not_redis::{Client, RedisResult};

async fn setup_client() -> Client {
    let client = Client::new();
    client.start().await;
    client
}

async fn cleanup(client: &mut Client) {
    let _ = client.flushdb().await;
}

mod string_tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_get() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, "value1");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_set_overwrite() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.set("key1", "value2").await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, "value2");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let mut client = setup_client().await;
        let result: String = client.get("nonexistent").await.unwrap();
        assert_eq!(result, "");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_del_single() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        let deleted: i64 = client.del("key1").await.unwrap();
        assert_eq!(deleted, 1);
        let exists: bool = client.exists("key1").await.unwrap();
        assert!(!exists);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_del_nonexistent() {
        let mut client = setup_client().await;
        let deleted: i64 = client.del("nonexistent").await.unwrap();
        assert_eq!(deleted, 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_exists_true() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        let exists: bool = client.exists("key1").await.unwrap();
        assert!(exists);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_exists_false() {
        let mut client = setup_client().await;
        let exists: bool = client.exists("nonexistent").await.unwrap();
        assert!(!exists);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_expire_basic() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        let result: bool = client.expire("key1", 60).await.unwrap();
        assert!(result);
        let ttl: i64 = client.ttl("key1").await.unwrap();
        assert!(ttl > 0 && ttl <= 60);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_ttl_positive() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.expire("key1", 120).await.unwrap();
        let ttl: i64 = client.ttl("key1").await.unwrap();
        assert!(ttl > 0 && ttl <= 120);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_ttl_nonexistent() {
        let mut client = setup_client().await;
        let ttl: i64 = client.ttl("nonexistent").await.unwrap();
        assert_eq!(ttl, -2);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_persist_removes_expiry() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.expire("key1", 60).await.unwrap();
        let persisted: bool = client.persist("key1").await;
        assert!(persisted);
        let ttl: i64 = client.ttl("key1").await.unwrap();
        assert_eq!(ttl, -1); // -1 means no expiry
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_type_conversion_string() {
        let mut client = setup_client().await;
        let test_str = "hello world";
        client.set("key1", test_str).await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, test_str);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_type_conversion_i64() {
        let mut client = setup_client().await;
        let test_num: i64 = 42;
        client.set("key1", test_num).await.unwrap();
        let result: i64 = client.get("key1").await.unwrap();
        assert_eq!(result, 42);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_type_conversion_bool() {
        let mut client = setup_client().await;
        client.set("key1", true).await.unwrap();
        let result: bool = client.get("key1").await.unwrap();
        assert!(result);
        cleanup(&mut client).await;
    }
}

mod hash_tests {
    use super::*;

    #[tokio::test]
    async fn test_hset_new_field() {
        let mut client = setup_client().await;
        let result: i64 = client.hset("myhash", "field1", "value1").await.unwrap();
        assert_eq!(result, 1);
        let val: String = client.hget("myhash", "field1").await.unwrap();
        assert_eq!(val, "value1");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hset_existing_field() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        let result: i64 = client.hset("myhash", "field1", "value2").await.unwrap();
        assert_eq!(result, 0);
        let val: String = client.hget("myhash", "field1").await.unwrap();
        assert_eq!(val, "value2");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hset_creates_hash() {
        let mut client = setup_client().await;
        let result: i64 = client.hset("newkey", "field1", "value1").await.unwrap();
        assert_eq!(result, 1);
        let exists: bool = client.exists("newkey").await.unwrap();
        assert!(exists);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hget_existing() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        client.hset("myhash", "field2", "value2").await.unwrap();
        let val: String = client.hget("myhash", "field1").await.unwrap();
        assert_eq!(val, "value1");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hget_nonexistent() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        let val: String = client.hget("myhash", "nonexistent").await.unwrap();
        assert_eq!(val, "");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hgetall_multiple() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        client.hset("myhash", "field2", "value2").await.unwrap();
        client.hset("myhash", "field3", "value3").await.unwrap();
        let result: Vec<String> = client.hgetall("myhash").await.unwrap();
        assert_eq!(result.len(), 6);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hgetall_empty() {
        let mut client = setup_client().await;
        client.set("stringkey", "value").await.unwrap();
        let result: Vec<String> = client.hgetall("nonexistent").await.unwrap();
        assert_eq!(result.len(), 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hdel_existing() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        let result: i64 = client.hdel("myhash", "field1").await.unwrap();
        assert_eq!(result, 1);
        let val: String = client.hget("myhash", "field1").await.unwrap();
        assert_eq!(val, "");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hdel_nonexistent() {
        let mut client = setup_client().await;
        client.hset("myhash", "field1", "value1").await.unwrap();
        let result: i64 = client.hdel("myhash", "nonexistent").await.unwrap();
        assert_eq!(result, 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_wrong_type_on_string_key() {
        let mut client = setup_client().await;
        client.set("mykey", "stringvalue").await.unwrap();
        let result = client.hset("mykey", "field", "value").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }
}

mod list_tests {
    use super::*;

    #[tokio::test]
    async fn test_lpush_new() {
        let mut client = setup_client().await;
        let len: i64 = client.lpush("mylist", "first").await.unwrap();
        assert_eq!(len, 1);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_lpush_multiple() {
        let mut client = setup_client().await;
        client.lpush("mylist", "a").await.unwrap();
        client.lpush("mylist", "b").await.unwrap();
        client.lpush("mylist", "c").await.unwrap();
        let len: i64 = client.llen("mylist").await.unwrap();
        assert_eq!(len, 3);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_rpush_new() {
        let mut client = setup_client().await;
        let len: i64 = client.rpush("mylist", "first").await.unwrap();
        assert_eq!(len, 1);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_llen_basic() {
        let mut client = setup_client().await;
        client.lpush("mylist", "a").await.unwrap();
        client.lpush("mylist", "b").await.unwrap();
        let len: i64 = client.llen("mylist").await.unwrap();
        assert_eq!(len, 2);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_llen_empty() {
        let mut client = setup_client().await;
        let len: i64 = client.llen("nonexistent").await.unwrap();
        assert_eq!(len, 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_wrong_type_on_string_key() {
        let mut client = setup_client().await;
        client.set("mykey", "stringvalue").await.unwrap();
        let result = client.llen("mykey").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }
}

mod set_tests {
    use super::*;

    #[tokio::test]
    async fn test_sadd_new() {
        let mut client = setup_client().await;
        let result: i64 = client.sadd("myset", "member1").await.unwrap();
        assert_eq!(result, 1);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_sadd_duplicate() {
        let mut client = setup_client().await;
        client.sadd("myset", "member1").await.unwrap();
        let result: i64 = client.sadd("myset", "member1").await.unwrap();
        assert_eq!(result, 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_smembers_basic() {
        let mut client = setup_client().await;
        client.sadd("myset", "member1").await.unwrap();
        client.sadd("myset", "member2").await.unwrap();
        client.sadd("myset", "member3").await.unwrap();
        let members: Vec<String> = client.smembers("myset").await.unwrap();
        assert_eq!(members.len(), 3);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_smembers_empty() {
        let mut client = setup_client().await;
        let members: Vec<String> = client.smembers("nonexistent").await.unwrap();
        assert_eq!(members.len(), 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_wrong_type_on_string_key() {
        let mut client = setup_client().await;
        client.set("mykey", "stringvalue").await.unwrap();
        let result = client.sadd("mykey", "member").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }
}

mod utility_tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let mut client = setup_client().await;
        let result: String = client.ping().await.unwrap();
        assert_eq!(result, "PONG");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_echo() {
        let mut client = setup_client().await;
        let result: String = client.echo("hello world").await.unwrap();
        assert_eq!(result, "hello world");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_dbsize_empty() {
        let mut client = setup_client().await;
        let size: i64 = client.dbsize().await.unwrap();
        assert_eq!(size, 0);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_dbsize_after_ops() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.set("key2", "value2").await.unwrap();
        client.hset("hash1", "field1", "val1").await.unwrap();
        let size: i64 = client.dbsize().await.unwrap();
        assert_eq!(size, 3);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_flushdb() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.set("key2", "value2").await.unwrap();
        let _: String = client.flushdb().await.unwrap();
        let size: i64 = client.dbsize().await.unwrap();
        assert_eq!(size, 0);
        cleanup(&mut client).await;
    }
}

mod expiration_tests {
    use super::*;

    #[tokio::test]
    async fn test_expire_sets_ttl() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.expire("key1", 30).await.unwrap();
        let ttl: i64 = client.ttl("key1").await.unwrap();
        assert!(ttl > 0 && ttl <= 30);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_persist_preserves_key() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.expire("key1", 10).await.unwrap();
        let persisted = client.persist("key1").await;
        assert!(persisted);
        let ttl: i64 = client.ttl("key1").await.unwrap();
        assert_eq!(ttl, -1); // -1 means no expiry
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_ttl_mixed_expiry() {
        let mut client = setup_client().await;
        client.set("key1", "value1").await.unwrap();
        client.set("key2", "value2").await.unwrap();
        client.expire("key1", 60).await.unwrap();
        let ttl1: i64 = client.ttl("key1").await.unwrap();
        let ttl2: i64 = client.ttl("key2").await.unwrap();
        assert!(ttl1 > 0 && ttl1 <= 60);
        assert_eq!(ttl2, -1); // key2 has no expiry, so -1
        cleanup(&mut client).await;
    }
}

mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_string_key() {
        let mut client = setup_client().await;
        client.set("", "value1").await.unwrap();
        let result: String = client.get("").await.unwrap();
        assert_eq!(result, "value1");
        let deleted: i64 = client.del("").await.unwrap();
        assert_eq!(deleted, 1);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_empty_string_value() {
        let mut client = setup_client().await;
        client.set("key1", "").await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, "");
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_unicode_strings() {
        let mut client = setup_client().await;
        let unicode_str = "你好世界 🌍 Привет мир";
        client.set("key1", unicode_str).await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, unicode_str);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_binary_data() {
        let mut client = setup_client().await;
        let binary: Vec<u8> = vec![0x00, 0xFF, 0x42, 0x13, 0x37];
        client.set("key1", binary.clone()).await.unwrap();
        let result: Vec<u8> = client.get("key1").await.unwrap();
        assert_eq!(result, binary);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_special_characters() {
        let mut client = setup_client().await;
        let special = "key:value\nwith\ttabs\"and\"quotes\\backslashes";
        client.set("key1", special).await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, special);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_whitespace_values() {
        let mut client = setup_client().await;
        let whitespace = "   spaces   \n\tnewlines\t ";
        client.set("key1", whitespace).await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, whitespace);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_large_value() {
        let mut client = setup_client().await;
        let large = "x".repeat(10000);
        client.set("key1", large).await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result.len(), 10000);
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_numeric_strings() {
        let mut client = setup_client().await;
        client.set("key1", "12345").await.unwrap();
        let result: String = client.get("key1").await.unwrap();
        assert_eq!(result, "12345");
        cleanup(&mut client).await;
    }
}

mod error_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_wrong_type() {
        let mut client = setup_client().await;
        client.lpush("mylist", "item").await.unwrap();
        let result: RedisResult<String> = client.get("mylist").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_hget_wrong_type() {
        let mut client = setup_client().await;
        client.set("mykey", "stringvalue").await.unwrap();
        let result: RedisResult<String> = client.hget("mykey", "field").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_llen_wrong_type() {
        let mut client = setup_client().await;
        client.set("mykey", "value").await.unwrap();
        let result: RedisResult<i64> = client.llen("mykey").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_sadd_wrong_type() {
        let mut client = setup_client().await;
        client.set("mykey", "value").await.unwrap();
        let result: RedisResult<i64> = client.sadd("mykey", "member").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }

    #[tokio::test]
    async fn test_wrong_type_hset_on_list() {
        let mut client = setup_client().await;
        client.lpush("mylist", "item").await.unwrap();
        let result: RedisResult<i64> = client.hset("mylist", "field", "value").await;
        assert!(result.is_err());
        cleanup(&mut client).await;
    }
}
