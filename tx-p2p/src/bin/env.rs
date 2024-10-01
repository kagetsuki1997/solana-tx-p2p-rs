use crate::prefixed_string_constant;

prefixed_string_constant! {
    PREFIX = "TX_P2P_";

    pub const GRPC_ADDRESS;
    pub const GRPC_PORT;

    pub const METRICS_ADDRESS;
    pub const METRICS_PORT;

    pub const API_ADDRESS;
    pub const API_PORT;

    pub const TLS_CERT;
    pub const TLS_KEY;
    pub const TLS_CA;

    pub const MESSAGE_DURATION;
    pub const RELAY_LEADER_DURATION;
    pub const SIGNING_LEADER_DURATION;
}

#[macro_export]
macro_rules! prefixed_string_constant {
    (
        PREFIX = $prefix:expr;
        $($(#[$attr:meta])* $vis:vis const $name:ident;)*
    ) => {
        $(
            $(#[$attr])*
            $vis const $name: &'static str = concat!($prefix, stringify!($name));
        )*
    };
}
