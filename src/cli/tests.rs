use super::*;

use clap::CommandFactory;

#[test]
fn clap_test() {
    Arguments::command().debug_assert()
}
