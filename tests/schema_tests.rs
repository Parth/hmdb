#[cfg(test)]
pub mod schema {
    use hmdb::transaction::Transaction;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Instant;
    use uuid::Uuid;

    // Could make a type which holds a db and then on drop will clean up the db file.
    fn test_db() -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("target");
        path.push(Uuid::new_v4().to_string());
        path.to_str().unwrap().into()
    }

    #[test]
    fn start_empty() {
        // TODO: Uncomment to discover a namespacing issue
        // hmdb::schema! {
        //     Schema2 {
        //         word_counts: <String, u64>,
        //         word_counts2: <String, u64>
        //     }
        // }

        let db_path = &test_db();

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db = Schema::init(db_path).unwrap();

        assert!(!db.incomplete_write);

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    hmdb::schema! {
        Schema {
            word_counts: <String, u64>,
            word_counts2: <String, u8>
        }
    }

    // TODO maybe use this for function names: https://docs.rs/function_name/latest/function_name/
    #[test]
    fn write_test() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();
        db1.word_counts.insert("test".into(), 5).unwrap();

        let db2 = Schema::init(db_path).unwrap();
        assert_eq!(db2.word_counts.get(&"test".into()).unwrap().unwrap(), 5);
        db2.word_counts.insert("test2".into(), 3).unwrap();
        assert_eq!(db2.word_counts.get(&"test2".into()).unwrap().unwrap(), 3);

        let db3 = Schema::init(db_path).unwrap();
        assert_eq!(db3.word_counts.get(&"test2".into()).unwrap().unwrap(), 3);

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    #[test]
    fn exists_tests() {
        let db_path = &test_db();
        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();
        assert!(!db1.word_counts.exists(&"test".into()).unwrap());
        assert!(db1.word_counts.get(&"test".into()).unwrap().is_none());

        db1.word_counts.insert("test".into(), 435).unwrap();
        assert!(db1.word_counts.exists(&"test".into()).unwrap());
        assert!(db1.word_counts.get(&"test".into()).unwrap().is_some());

        db1.word_counts.delete("test".into()).unwrap();
        assert!(!db1.word_counts.exists(&"test".into()).unwrap());
        assert!(db1.word_counts.get(&"test".into()).unwrap().is_none());

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    #[test]
    fn delete_test_1() {
        let db_path = &test_db();
        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();
        db1.word_counts.insert("test".into(), 234).unwrap();
        db1.word_counts.delete("test".into()).unwrap();
        assert!(db1.word_counts.get(&"test".into()).unwrap().is_none());

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    #[test]
    fn delete_test_2() {
        let db_path = &test_db();
        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();
        db1.word_counts.insert("test".into(), 234).unwrap();

        let db2 = Schema::init(db_path).unwrap();
        db2.word_counts.delete("test".into()).unwrap();

        let db2 = Schema::init(db_path).unwrap();
        db2.word_counts.delete("test".into()).unwrap();

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    #[test]
    fn transaction_test_1() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();

        db1.word_counts.insert("test".to_string(), 5).unwrap();

        db1.transaction(|db| {
            let mut num = *db.word_counts.get(&"test".to_string()).unwrap();
            num += 1;
            db.word_counts.insert("test".to_string(), num).unwrap();
        })
        .unwrap();

        assert_eq!(
            db1.word_counts.get(&"test".to_string()).unwrap().unwrap(),
            6
        );

        let db2 = Schema::init(db_path).unwrap();
        assert_eq!(
            db2.word_counts.get(&"test".to_string()).unwrap().unwrap(),
            6
        );

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }

    #[test]
    fn transaction_test_2() {
        let db_path = &test_db();

        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));

        let db1 = Schema::init(db_path).unwrap();

        db1.word_counts.insert("test".to_string(), 5).unwrap();

        let thread_db = db1.clone();
        std::thread::spawn(move || {
            thread_db
                .transaction(|db| {
                    std::thread::sleep(Duration::from_secs(1));
                    let mut num = *db.word_counts.get(&"test".to_string()).unwrap();
                    num += 1;
                    db.word_counts.insert("test".to_string(), num).unwrap();
                })
                .unwrap();
        });
        std::thread::sleep(Duration::from_millis(20));
        let now = Instant::now();
        assert_eq!(
            db1.word_counts.get(&"test".to_string()).unwrap().unwrap(),
            6
        );
        assert!(now.elapsed().as_millis() > 800);
        fs::remove_dir_all(db_path)
            .unwrap_or_else(|err| println!("starting log did not exist: {}", err));
    }
}
