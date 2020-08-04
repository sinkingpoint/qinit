#[test]
fn test_passwd_parses_shadow_correctly() {
    let entry = match libq::passwd::ShadowEntry::from_shadow_line(&"colin:$6$e6.TbOaDtIkSRV0B$D3uQD.IcZJZqfFL1pE7mwnvR2LaD.RfUk/BIWaPO1jPkXuv/gNit8Tkr2uveGvnuMeWEZxfT9ViN7F6XlqT3d0:18353:0:99999:7:::".to_owned()) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("{}", err);
            assert!(false);
            return;
        }
    };

    assert_eq!(entry.username, "colin");
    assert_eq!(entry.day_of_last_change, 18353);
    assert_eq!(entry.min_time_days, 0);
    assert_eq!(entry.max_time_days, 99999);
    assert_eq!(entry.warn_time_days, 7);
    assert_eq!(entry.inactive_time_days, 0);
    assert_eq!(entry.expire_time_days, 0);

    assert_eq!(entry.password_hash.algorithm, libq::passwd::PasswdAlgorithm::SHA512);
    assert_eq!(entry.password_hash.salt, "e6.TbOaDtIkSRV0B");
    assert_eq!(
        entry.password_hash.hash,
        "D3uQD.IcZJZqfFL1pE7mwnvR2LaD.RfUk/BIWaPO1jPkXuv/gNit8Tkr2uveGvnuMeWEZxfT9ViN7F6XlqT3d0"
    );
}

#[test]
fn test_passwd_validates_sha256_hashes() {
    let test_cases = [
        ("Hello world!", "$5$saltstring$5B8vYYiY.CVt1RlTTf8KbXBH3hsxY/GNooZaBBGWEc5"),
        (
            "Hello world!",
            "$5$rounds=10000$saltstringsaltst$3xv.VbSHBb41AL9AvLeujZkZRBAwqFMz2.opqey6IcA",
        ),
        (
            "This is just a test",
            "$5$rounds=5000$toolongsaltstrin$Un/5jzAHMgOGZ5.mWJpuVolil07guHPvOW8mGRcvxa5",
        ),
        (
            "a very much longer text to encrypt.  This one even stretches over morethan one line.",
            "$5$rounds=1400$anotherlongsalts$Rx.j8H.h8HjEDGomFU8bDkXm3XIUnzyxf12oP84Bnq1",
        ),
        (
            "we have a short salt string but not a short password",
            "$5$rounds=77777$short$JiO1O3ZpDAxGJeaDIuqCoEFysAe1mZNJRs3pw0KQRd/",
        ),
        (
            "a short string",
            "$5$rounds=123456$asaltof16chars..$gP3VQ/6X7UUEW3HkBn2w1/Ptq2jxPyzV/cZKmF/wJvD",
        ),
        (
            "the minimum number is still observed",
            "$5$rounds=1000$roundstoolow$yfvwcWrQ8l/K0DAWyuPMDNHpIVlTQebY9l/gL972bIC",
        ),
    ];

    for test_case in test_cases.into_iter() {
        let (password, expected_hash) = test_case;

        let hash = libq::passwd::UnixPasswordHash::from_unix_hash_str(expected_hash);
        assert!(hash.is_some(), "Expected {} to be parsed correctly", expected_hash);
        let hash = hash.unwrap();
        assert_eq!(hash.algorithm, libq::passwd::PasswdAlgorithm::SHA256);
        assert!(hash.verify_str(password), "Expected {} to hash into {}", password, expected_hash);
    }
}

#[test]
fn test_passwd_validates_sha512_hashes() {
    let test_cases = [
        (
            "Hello world!",
            "$6$saltstring$svn8UoSVapNtMuq1ukKS4tPQd8iKwSMHWjl/O817G3uBnIFNjnQJuesI68u4OTLiBFdcbYEdFCoEOfaS35inz1",
        ),
        (
            "Hello world!",
            "$6$rounds=10000$saltstringsaltst$OW1/O6BYHV6BcXZu8QVeXbDWra3Oeqh0sbHbbMCVNSnCM/UrjmM0Dp8vOuZeHBy/YTBmSK6H9qs/y3RnOaw5v.",
        ),
        (
            "This is just a test",
            "$6$rounds=5000$toolongsaltstrin$lQ8jolhgVRVhY4b5pZKaysCLi0QBxGoNeKQzQ3glMhwllF7oGDZxUhx1yxdYcz/e1JSbq3y6JMxxl8audkUEm0",
        ),
        (
            "a very much longer text to encrypt.  This one even stretches over morethan one line.",
            "$6$rounds=1400$anotherlongsalts$POfYwTEok97VWcjxIiSOjiykti.o/pQs.wPvMxQ6Fm7I6IoYN3CmLs66x9t0oSwbtEW7o7UmJEiDwGqd8p4ur1",
        ),
        (
            "we have a short salt string but not a short password",
            "$6$rounds=77777$short$WuQyW2YR.hBNpjjRhpYD/ifIw05xdfeEyQoMxIXbkvr0gge1a1x3yRULJ5CCaUeOxFmtlcGZelFl5CxtgfiAc0",
        ),
        (
            "a short string",
            "$6$rounds=123456$asaltof16chars..$BtCwjqMJGx5hrJhZywWvt0RLE8uZ4oPwcelCjmw2kSYu.Ec6ycULevoBK25fs2xXgMNrCzIMVcgEJAstJeonj1",
        ),
        (
            "the minimum number is still observed",
            "$6$rounds=1000$roundstoolow$kUMsbe306n21p9R.FRkW3IGn.S9NPN0x50YhH1xhLsPuWGsUSklZt58jaTfF4ZEQpyUNGc0dqbpBYYBaHHrsX.",
        ),
    ];

    for test_case in test_cases.into_iter() {
        let (password, expected_hash) = test_case;

        let hash = libq::passwd::UnixPasswordHash::from_unix_hash_str(expected_hash);
        assert!(hash.is_some(), "Expected {} to be parsed correctly", expected_hash);
        let hash = hash.unwrap();
        assert_eq!(hash.algorithm, libq::passwd::PasswdAlgorithm::SHA512);
        assert!(hash.verify_str(password), "Expected {} to hash into {}", password, expected_hash);
    }
}
