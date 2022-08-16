use std::{
    thread::{self, sleep},
    time,
};

use blockchain::{
    node::{create_debug_node, create_full_node, create_genesis_node},
    user::User,
};

fn main() {
    thread::spawn(|| {
        let user = User::from_nothingness();
        create_genesis_node(
            "127.0.0.1:5333".to_owned(),
            ["127.0.0.1:5334".to_owned(), "127.0.0.1:5335".to_owned()].to_vec(),
            user.as_public(),
        );
    });

    thread::spawn(|| {
        let user = User::from_nothingness();
        create_full_node(
            "127.0.0.1:5334".to_owned(),
            ["127.0.0.1:5333".to_owned(), "127.0.0.1:5335".to_owned()].to_vec(),
            user.as_public(),
        );
    });

    thread::spawn(|| {
        create_debug_node(
            "127.0.0.1:5335".to_owned(),
            ["127.0.0.1:5333".to_owned(), "127.0.0.1:5334".to_owned()].to_vec(),
        );
    });

    sleep(time::Duration::from_secs(10));
}
