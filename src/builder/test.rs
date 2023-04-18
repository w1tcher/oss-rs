use super::ArcPointer;

#[test]
fn debug_arc_pointer() {
    let pointer = ArcPointer;

    assert_eq!(format!("{pointer:?}"), "ArcPointer");
}

#[test]
#[cfg(feature = "blocking")]
fn debug_rc_pointer() {
    use super::RcPointer;
    let pointer = RcPointer;

    assert_eq!(format!("{pointer:?}"), "RcPointer");
}

mod error {
    use std::error::Error;

    use crate::{
        builder::{BuilderError, BuilderErrorKind},
        config::InvalidConfig,
        errors::OssService,
        types::InvalidEndPoint,
    };

    #[tokio::test]
    async fn from_reqwest() {
        use http::response::Builder;
        use reqwest::Response;
        use serde::Deserialize;

        let response = Builder::new().status(200).body("aaaa").unwrap();

        #[derive(Debug, Deserialize)]
        struct Ip;

        let response = Response::from(response);
        let err = response.json::<Ip>().await.unwrap_err();

        let builder_err = BuilderError::from(err);
        assert_eq!(format!("{builder_err}"), "reqwest error");
        assert_eq!(
            format!("{}", builder_err.source().unwrap()),
            "error decoding response body: expected value at line 1 column 1"
        );
        assert_eq!(format!("{:?}", builder_err), "BuilderError { kind: Reqwest(reqwest::Error { kind: Decode, source: Error(\"expected value\", line: 1, column: 1) }) }");
    }

    #[test]
    fn from_oss() {
        let err = BuilderError {
            kind: BuilderErrorKind::OssService(OssService::default()),
        };
        assert_eq!(format!("{err}"), "http status is not success");
        assert_eq!(format!("{}", err.source().unwrap()), "OssService { code: \"Undefined\", status: 200, message: \"Parse aliyun response xml error message failed.\", request_id: \"XXXXXXXXXXXXXXXXXXXXXXXX\" }");

        fn bar() -> BuilderError {
            OssService::default().into()
        }

        assert_eq!(format!("{:?}", bar()), "BuilderError { kind: OssService(OssService { code: \"Undefined\", status: 200, message: \"Parse aliyun response xml error message failed.\", request_id: \"XXXXXXXXXXXXXXXXXXXXXXXX\" }) }")
    }

    #[test]
    #[cfg(feature = "auth")]
    fn from_auth() {
        use std::error::Error;

        use crate::{
            auth::{AuthError, AuthErrorKind},
            builder::BuilderErrorKind,
        };

        let err = BuilderError {
            kind: BuilderErrorKind::Auth(AuthError {
                kind: AuthErrorKind::InvalidCanonicalizedResource,
            }),
        };
        assert_eq!(format!("{err}"), "aliyun auth failed");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "invalid canonicalized-resource"
        );

        fn bar() -> BuilderError {
            AuthError {
                kind: AuthErrorKind::InvalidCanonicalizedResource,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "BuilderError { kind: Auth(AuthError { kind: InvalidCanonicalizedResource }) }"
        );
    }

    #[test]
    fn from_config() {
        let err = BuilderError {
            kind: BuilderErrorKind::Config(InvalidConfig::EndPoint(InvalidEndPoint { _priv: () })),
        };
        assert_eq!(format!("{err}"), "oss config error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        fn bar() -> BuilderError {
            InvalidConfig::EndPoint(InvalidEndPoint { _priv: () }).into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "BuilderError { kind: Config(EndPoint(InvalidEndPoint)) }"
        );
    }
}
