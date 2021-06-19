  let root_key: AccountId = hex!["12970155d02df21b7e39e289593065d0bbb67d5d38f36dd1b9d617614a006d00"].into(); // 5znMeMdGsDrENMFg9wvLMveuYdCSVCCGdaXE6HAU4UwTksei
  let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
    (
      hex!["d0a9b0c9ac0a3dc0432cb66f288c1ffc9bd159ca52739d994f789003b08b6630"].into(),              // 655aHzD3sX1QpZVxStEHPV4TVCqKVcfwfxrsX8spZndPfabe
      hex!["c43b6cda18d09359fe32ea27014601c6d723e17e2cc8ca14496f210595f95a26"].into(),              // 64oGxqAX2AW26AWQDx9vNNb7aTF741QMTn1n35qFRty6FaLc
      hex!["184f5672c5f405f12476c29ba35ab22fdf44f4e50d671802cb271f06adb5cb3f"].unchecked_into(),    // 5zureDa91LCdspDmqxkPUnGg9WLHPJQLs1XZ9uqmkUEcK3Ca
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
    ),
    (
      hex!["5e7704ab35a8a08fda1ca9ddca87013849daf02744e81cc5fb03d7395030744c"].into(),              // 62VqnJu5Xwc5qaNsQoeS8UAEA8rFFf8U6UeyeKgYQGfi23us
      hex!["c23b0e2abab64d27c630028830d5a3afc4785f0dd02ce069af8b3f2118bc682c"].into(),              // 64kekuPLYqkAHwwbeYjVUDkPFoc27VNGib3ezJrXCTY2qWSm
      hex!["b46c28b4f0db186814fe579e63d2e9b7c3dbb6c1f28dfe541a6cc11ccfc5fa3e"].unchecked_into(),    // 64SYg4L1MbtsREC8Qcrd42bMidA8bXq9jmNBYDwAg1fcuBm4
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
    ),
  ];
