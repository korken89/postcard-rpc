/// ## Endpoint macro
///
/// Used to define a single Endpoint marker type that implements the
/// [Endpoint][crate::Endpoint] trait.
///
/// ```rust
/// # use postcard::experimental::schema::Schema;
/// # use serde::{Serialize, Deserialize};
/// use postcard_rpc::endpoint;
///
/// #[derive(Debug, Serialize, Deserialize, Schema)]
/// pub struct Req1 {
///     a: u8,
///     b: u64,
/// }
///
/// #[derive(Debug, Serialize, Deserialize, Schema)]
/// pub struct Resp1 {
///     c: [u8; 4],
///     d: i32,
/// }
///
/// endpoint!(Endpoint1, Req1, Resp1, "endpoint/1");
/// ```
///
/// If the path is omitted, the type name is used instead.
#[macro_export]
macro_rules! endpoint {
    ($tyname:ident, $req:ty, $resp:ty) => {
        endpoint!($tyname, $req, $resp, stringify!($tyname));
    };
    ($tyname:ident, $req:ty, $resp:ty, $path:expr,) => {
        endpoint!($tyname, $req, $resp, $path)
    };
    ($tyname:ident, $req:ty, $resp:ty, $path:expr) => {
        pub struct $tyname;

        impl $crate::Endpoint for $tyname {
            type Request = $req;
            type Response = $resp;
            const PATH: &'static str = $path;
            const REQ_KEY: $crate::Key = $crate::Key::for_path::<$req>($path);
            const RESP_KEY: $crate::Key = $crate::Key::for_path::<$resp>($path);
        }
    };
}

/// ## Topic macro
///
/// Used to define a single Topic marker type that implements the
/// [Topic][crate::Topic] trait.
///
/// ```rust
/// # use postcard::experimental::schema::Schema;
/// # use serde::{Serialize, Deserialize};
/// use postcard_rpc::topic;
///
/// #[derive(Debug, Serialize, Deserialize, Schema)]
/// pub struct Message1 {
///     a: u8,
///     b: u64,
/// }
///
/// topic!(Topic1, Message1, "topic/1");
/// ```
///
/// If the path is omitted, the type name is used instead.
#[macro_export]
macro_rules! topic {
    ($tyname:ident, $msg:ty) => {
        topic!($tyname, $msg, stringify!($tyname));
    };
    ($tyname:ident, $msg:ty, $path:expr,) => {
        topic!($tyname, $msg, $path)
    };
    ($tyname:ident, $msg:ty, $path:expr) => {
        pub struct $tyname;

        impl $crate::Topic for $tyname {
            type Message = $msg;
            const PATH: &'static str = $path;
            const TOPIC_KEY: $crate::Key = $crate::Key::for_path::<$msg>($path);
        }
    };
}

/// ## Dispatch macro
///
/// Helps when implementing dispatch on embedded systems.
///
/// ```rust
/// // TODO
/// ```
#[macro_export]
macro_rules! dispatch {
    (
        $buf:ident,
        $unhandled:pat = _ => $unhandled_body:tt,
        $(EP: $ep_request:pat = $endpoint:path => $ep_body:tt),*
        $(TP: $topic_pl:pat = $topic:path => $topic_body:tt),*
    ) => {
    {
        const _UNIQ: () = {
            let keys = [$(<$endpoint as $crate::Endpoint>::REQ_KEY),* $(<$topic as $crate::Topic>::TOPIC_KEY),*];

            let mut i = 0;

            while i < keys.len() {
                let mut j = i + 1;
                while j < keys.len() {
                    if keys[i].const_cmp(&keys[j]) {
                        panic!("Keys are not unique, there is a collision!");
                    }
                    j += 1;
                }

                i += 1;
            }
        };

        let _ = _UNIQ;

        // TODO: const assert that all keys are unique
        match $crate::headered::extract_header_from_bytes($buf) {
            Ok((hdr, body)) => {
                match hdr.key {
                $(
                    <$endpoint as $crate::Endpoint>::REQ_KEY => {
                        match $crate::export::postcard::take_from_bytes::<<$endpoint as $crate::Endpoint>::Request>(body) {
                            Ok((req, _rest)) => {
                                let $ep_request = (&hdr, req);
                                $ep_body

                                Ok(())
                            }
                            Err(e) => Err($crate::DispatchError::Body(e))
                        }
                    }
                )*
                $(
                    <$topic as $crate::Topic>::TOPIC_KEY => {
                        match $crate::export::postcard::take_from_bytes::<<$topic as $crate::Topic>::Message>(body) {
                            Ok((msg, _rest)) => {
                                let $topic_pl = (&hdr, msg);
                                $topic_body

                                Ok(())
                            }
                            Err(e) => Err($crate::DispatchError::Body(e))
                        }
                    }
                )*
                    _ => {
                        let $unhandled = (&hdr, body);

                        $unhandled_body

                        Ok(())
                    }
                }
            }
            Err(e) => Err($crate::DispatchError::Header(e)),
        }
    }
};
}
