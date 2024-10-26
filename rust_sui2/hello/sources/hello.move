module hello::hello {
    use sui::event;
    use std::string::{Self, String};

    public struct HelloEvent has copy, drop {
        say: String
    }

    entry fun hello() {
        let say = string::utf8(b"Hello! Thanks for using Rust to publish and call Sui move!");
        event::emit(HelloEvent { say });
    }
}