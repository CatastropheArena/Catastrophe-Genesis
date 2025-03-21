#[test_only]
module nexus::nexus_tests;
// uncomment this line to import the module
// use nexus::nexus;

const ENotImplemented: u64 = 0;

#[test]
fun test_nexus() {
    // pass
}

#[test, expected_failure(abort_code = ::nexus::nexus_tests::ENotImplemented)]
fun test_nexus_fail() {
    abort ENotImplemented
}
