
//extern crate base64;

use std::convert::TryInto;

use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName, CONTENT_TYPE};
use crate::types::{KeyId,KeySecret,ContentMd5,CanonicalizedResource, Date, ContentType};
use crate::errors::{OssResult, OssError};
// use http::Method;
// #[cfg(test)]
// use mockall::{automock, mock, predicate::*};

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
#[non_exhaustive]
pub struct VERB(pub Method);

#[derive(Default, Clone)]
pub struct Auth{
  pub access_key_id: KeyId,
  pub access_key_secret: KeySecret,
  pub verb: VERB,
  pub content_md5: Option<ContentMd5>,
  pub content_type: Option<ContentType>,
  pub date: Date,
  // pub canonicalized_oss_headers: &'a str, // TODO
  pub canonicalized_resource: CanonicalizedResource,
  pub headers: HeaderMap,
}

impl VERB {
  /// GET
  pub const GET: VERB = VERB(Method::GET);

  /// POST
  pub const POST: VERB = VERB(Method::POST);

  /// PUT
  pub const PUT: VERB = VERB(Method::PUT);

  /// DELETE
  pub const DELETE: VERB = VERB(Method::DELETE);

  /// HEAD
  pub const HEAD: VERB = VERB(Method::HEAD);

  /// OPTIONS
  pub const OPTIONS: VERB = VERB(Method::OPTIONS);

  /// CONNECT
  pub const CONNECT: VERB = VERB(Method::CONNECT);

  /// PATCH
  pub const PATCH: VERB = VERB(Method::PATCH);

  /// TRACE
  pub const TRACE: VERB = VERB(Method::TRACE);

  #[inline]
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }
}

impl TryInto<HeaderValue> for VERB {
    type Error = OssError;
    fn try_into(self) -> OssResult<HeaderValue> {
        self.0.to_string().parse::<HeaderValue>()
        .map_err(|_| OssError::Input("VERB parse error".to_string()))
    }
}

impl From<VERB> for String {
  fn from(verb: VERB) -> Self {
    match verb.0 {
      Method::GET => "GET".into(),
      Method::POST => "POST".into(),
      Method::PUT => "PUT".into(),
      Method::DELETE => "DELETE".into(),
      Method::HEAD => "HEAD".into(),
      Method::OPTIONS => "OPTIONS".into(),
      Method::CONNECT => "CONNECT".into(),
      Method::PATCH => "PATCH".into(),
      Method::TRACE => "TRACE".into(),
      _ => "".into(),
    }
  }
}

impl From<&str> for VERB {
  fn from(str: &str) -> Self {
      match str {
          "POST"    => VERB(Method::POST),
          "GET"     => VERB(Method::GET),
          "PUT"     => VERB(Method::PUT),
          "DELETE"  => VERB(Method::DELETE),
          "HEAD"    => VERB(Method::HEAD),
          "OPTIONS" => VERB(Method::OPTIONS),
          "CONNECT" => VERB(Method::CONNECT),
          "PATCH"   => VERB(Method::PATCH),
          "TRACE"   => VERB(Method::TRACE),
          _ => VERB(Method::GET),
      }
  }
}

impl Default for VERB {
  fn default() -> Self {
      Self::GET
  }
}

type HmacSha1 = Hmac<Sha1>;

impl Auth {


  /// # 获取所有 header 信息
  /// 
  /// 包含 *公共 header*, *业务 header* 以及 **签名**
  #[cfg(feature = "blocking")]
  pub fn get_headers(self) -> OssResult<HeaderMap> {
    use futures::executor::block_on;
    block_on(self.async_get_headers())
  }

  pub async fn async_get_headers(&self) -> OssResult<HeaderMap> {
    let mut map= self.headers.clone();

    map.insert("AccessKeyId", self.access_key_id.as_ref().try_into()?);
    map.insert("SecretAccessKey", self.access_key_secret.as_ref().try_into()?);
    map.insert("VERB",self.verb.clone().try_into()?);

    if let Some(a) = self.content_md5.clone() {
      map.insert("Content-MD5",a.try_into()?);
    }
    if let Some(a) = &self.content_type {
      map.insert("Content-Type",a.as_ref().try_into()?);
    }
    map.insert("Date",self.date.as_ref().try_into()?);
    map.insert("CanonicalizedResource", self.canonicalized_resource.as_ref().try_into()?);

    let sign = self.sign()?;
    let sign = format!("OSS {}:{}", self.access_key_id, &sign);
    map.insert(
      "Authorization", 
      sign.parse().map_err(|_| OssError::Input("Authorization parse error".to_string()))?);

    //println!("header list: {:?}",map);
    Ok(map)
  }

  /// # 业务 header
  /// 
  /// 将 header 中除了共同部分的，转换成字符串，一般是 `x-oss-` 开头的
  /// 
  /// 用于生成签名 
  pub fn header_str(&self) -> OssResult<Option<String>> {
    //return Some("x-oss-copy-source:/honglei123/file1.txt");
    let mut header: Vec<(&HeaderName, &HeaderValue)> = self.headers.iter().filter(|(k,_v)|{
      k.as_str().starts_with("x-oss-")
    }).collect();
    if header.len()==0{
      return Ok(None);
    }

    header.sort_by(|(k1,_),(k2,_)| k1.to_string().cmp(&k2.to_string()));
    let header_vec: Vec<String> = header.into_iter().map(|(k,v)| -> OssResult<String> {
      let val = v.to_str().map_err(|e| OssError::ToStr(e.to_string()));

      let value = k.as_str().to_owned() + ":" 
        + val?;
      Ok(value)
    }).filter(|res|res.is_ok())
    // 这里的 unwrap 不会 panic
    .map(|res|res.unwrap())
    .collect();

    Ok(Some(header_vec.join("\n")))
  }

  /// 计算签名
  /// TODO 优化
  pub fn sign(&self) -> OssResult<String> {
    let method = self.verb.to_string();
    let mut content = String::new();

    let str: String = method
      + "\n"
      + match self.content_md5.as_ref() {
        Some(str)=> {
          str.as_ref()
        },
        None => ""
      }
      + "\n"
      + match &self.content_type {
        Some(str) => {
          str.as_ref()
        },
        None => ""
      }
      + "\n"
      + self.date.as_ref() 
      + "\n"
      + match self.header_str()? {
        Some(str) => {
          content.clear();
          content.push_str(&str);
          content.push_str("\n");
          &content
        },
        None => ""
      }
      + self.canonicalized_resource.as_ref();
    
    #[cfg(test)]
    println!("auth str: {}", str);
    
    let secret = self.access_key_secret.as_bytes();
    let str_u8 = str.as_bytes();
    
    let mut mac = HmacSha1::new_from_slice(secret)?;

    mac.update(str_u8);

    let sha1 = mac.finalize().into_bytes();

    Ok(encode(sha1))
  }

}

#[derive(Default, Clone)]
pub struct AuthBuilder{
  pub auth: Auth,
}

impl AuthBuilder{
  /// 给 key 赋值
  /// 
  /// ```
  /// use aliyun_oss_client::auth::AuthBuilder;
  /// 
  /// let mut builder = AuthBuilder::default();
  /// assert_eq!(builder.auth.access_key_id.as_ref(), "");
  /// builder = builder.key("bar");
  /// assert_eq!(builder.auth.access_key_id.as_ref(), "bar");
  /// ```
  pub fn key<K: Into<KeyId>>(mut self, key: K) -> Self {
    self.auth.access_key_id = key.into();
    self
  }

  /// 给 secret 赋值
  pub fn secret<K: Into<KeySecret>>(mut self, secret: K) -> Self {
    self.auth.access_key_secret = secret.into();
    self
  }

  /// 给 verb 赋值
  pub fn verb<T: Into<VERB>>(mut self, verb: T) -> Self {
    self.auth.verb = verb.into();
    self
  }

  /// 给 content_md5 赋值
  pub fn content_md5<M: Into<ContentMd5>>(mut self, content_md5: M) -> Self {
    self.auth.content_md5 = Some(content_md5.into());
    self
  }

  /// 给 date 赋值
  /// 
  /// example
  /// ```
  /// use chrono::Utc;
  /// let builder = aliyun_oss_client::auth::AuthBuilder::default()
  ///    .date(Utc::now());
  /// ```
  pub fn date<D: Into<Date>>(mut self, date: D) -> Self {
    self.auth.date = date.into();
    self
  }

  /// 给 content_md5 赋值
  pub fn canonicalized_resource<C: Into<CanonicalizedResource>>(mut self, data: C) -> Self {
    self.auth.canonicalized_resource = data.into();
    self
  }

  pub fn headers(mut self, headers: HeaderMap) -> Self {
    self.auth.headers = headers;
    self.type_with_header()
  }

  /// 给 header 序列添加新值
  pub fn header_insert<K: IntoHeaderName>(mut self, key: K, val: HeaderValue ) -> Self
  {
    self.auth.headers.insert(key, val);
    self
  }

  /// 通过 headers 给 content_type 赋值
  /// 
  /// TODO 需要处理异常的情况
  pub fn type_with_header(mut self) -> Self {
    let content_type = self.auth.headers.get(CONTENT_TYPE);

    if let Some(ct) = content_type {
      let t: OssResult<ContentType> = ct.clone().try_into();
      if let Ok(value) = t {
        self.auth.content_type = Some(value);
      }
    }
    self
  }

  /// 清理 headers
  pub fn header_clear(mut self) -> Self {
    self.auth.headers.clear();
    self
  }
}