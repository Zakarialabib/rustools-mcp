// cache_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_retrieve() {
        let mut cache = Cache::new();
        cache.insert("key1", "value1");
        assert_eq!(cache.retrieve("key1"), Some(&"value1"));
    }

    #[test]
    fn test_cache_persistence() {
        let mut cache = Cache::new();
        cache.insert("key2", "value2");
        cache.save_to_disk().unwrap(); // Assuming save_to_disk is implemented

        let loaded_cache = Cache::load_from_disk().unwrap(); // Assuming load_from_disk is implemented
        assert_eq!(loaded_cache.retrieve("key2"), Some(&"value2"));
    }
}