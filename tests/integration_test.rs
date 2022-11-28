mod test_async {
    use aliyun_oss_client::builder::ClientWithMiddleware;
    use aliyun_oss_client::client::Client;
    use aliyun_oss_client::file::File;
    use assert_matches::assert_matches;
    use dotenv::dotenv;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_bucket_list() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_list().await;

        assert_matches!(bucket_list, Ok(_));
    }

    #[tokio::test]
    async fn test_get_bucket_info() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_info().await;

        assert_matches!(bucket_list, Ok(_));
    }

    #[tokio::test]
    async fn get_object_by_bucket_struct() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_list().await.unwrap();

        let query = vec![("max-keys", "5"), ("prefix", "babel")];

        let buckets = bucket_list.buckets;
        let the_bucket = &buckets[0];
        let object_list = the_bucket.get_object_list(query).await;
        assert_matches!(object_list, Ok(_));
    }

    #[tokio::test]
    async fn test_get_object() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let object_list = client.get_object_list(vec![()]).await;

        assert_matches!(object_list, Ok(_));
    }

    #[tokio::test]
    async fn test_put_and_delete_file() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let object_list = client
            .put_file(
                PathBuf::from("examples/bg2015071010.png"),
                "examples/bg2015071010.png",
            )
            .await;

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png").await;

        assert_matches!(result, Ok(_));
    }
}

#[cfg(feature = "blocking")]
mod test_blocking {

    use aliyun_oss_client::blocking::builder::ClientWithMiddleware;
    use aliyun_oss_client::client::Client;
    use aliyun_oss_client::file::BlockingFile;
    use aliyun_oss_client::types::Query;
    use assert_matches::assert_matches;
    use dotenv::dotenv;
    use std::path::PathBuf;

    #[test]
    fn test_get_bucket_list() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_list();

        assert_matches!(bucket_list, Ok(_));
    }

    #[test]
    fn test_get_bucket_info() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_info();

        assert_matches!(bucket_list, Ok(_));
    }

    #[test]
    fn get_object_by_bucket_struct() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let bucket_list = client.get_bucket_list().unwrap();

        let buckets = bucket_list.buckets;
        let the_bucket = &buckets[0];
        let object_list = the_bucket.get_object_list(vec![("max-keys", "2")]);
        assert_matches!(object_list, Ok(_));
        let mut object_list = object_list.unwrap();
        assert_matches!(object_list.next(), Some(_));
    }

    #[test]
    fn test_get_object() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        let object_list = client.get_object_list(vec![()]);

        assert_matches!(object_list, Ok(_));
    }

    #[test]
    fn test_get_object_next() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();
        let query = vec![("max-keys", "2")];
        let mut object_list = client.get_object_list(query).unwrap();

        assert_matches!(object_list.next(), Some(_));
        assert_matches!(object_list.next(), Some(_));
    }

    #[test]
    fn test_put_and_delete_file() {
        dotenv().ok();

        let client = Client::<ClientWithMiddleware>::from_env().unwrap();

        // 第一种读取文件路径的方式
        let object_list = client.put_file(
            PathBuf::from("examples/bg2015071010.png"),
            "examples/bg2015071010.png",
        );

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png");

        assert_matches!(result, Ok(_));

        // 第二种读取文件路径的方式
        let object_list = client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png");

        assert_matches!(object_list, Ok(_));

        let result = client.delete_object("examples/bg2015071010.png");

        assert_matches!(result, Ok(_));
    }

    // #[bench]
    // fn bench_get_object(b: &mut Bencher){
    //   dotenv().ok();

    //   let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    //   let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    //   let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    //   let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    //   let client = client::Client::new(key_id,key_secret, endpoint, bucket);
    //   b.iter(|| {
    //     client.get_object_list();
    //   });
    // }
}
