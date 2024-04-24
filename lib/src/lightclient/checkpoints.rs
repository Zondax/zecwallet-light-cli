pub fn get_closest_checkpoint(
    chain_name: &str,
    height: u64,
) -> Option<(u64, &'static str, &'static str)> {
    log::info!("Trying to get checkpoint closest to block {}", height);
    match chain_name {
        "ztestsapling" => get_test_checkpoint(height),
        "zs" | "main" => get_main_checkpoint(height),
        _ => None,
    }
}

fn get_test_checkpoint(height: u64) -> Option<(u64, &'static str, &'static str)> {
    let checkpoints: Vec<(u64, &str, &str)> = vec![
        (600000, "0107385846c7451480912c294b6ce1ee1feba6c2619079fd9104f6e71e4d8fe7",
                 "01690698411e3f8badea7da885e556d7aba365a797e9b20b44ac0946dced14b23c001001ab2a18a5a86aa5d77e43b69071b21770b6fe6b3c26304dcaf7f96c0bb3fed74d000186482712fa0f2e5aa2f2700c4ed49ef360820f323d34e2b447b78df5ec4dfa0401a332e89a21afb073cb1db7d6f07396b56a95e97454b9bca5a63d0ebc575d3a33000000000001c9d3564eff54ebc328eab2e4f1150c3637f4f47516f879a0cfebdf49fe7b1d5201c104705fac60a85596010e41260d07f3a64f38f37a112eaef41cd9d736edc5270145e3d4899fcd7f0f1236ae31eafb3f4b65ad6b11a17eae1729cec09bd3afa01a000000011f8322ef806eb2430dc4a7a41c1b344bea5be946efc7b4349c1c9edb14ff9d39"
        ),
        (650000, "003f7e09a357a75c3742af1b7e1189a9038a360cebb9d55e158af94a1c5aa682",
                 "010113f257f93a40e25cfc8161022f21c06fa2bc7fb03ee9f9399b3b30c636715301ef5b99706e40a19596d758bf7f4fd1b83c3054557bf7fab4801985642c317d41100001b2ad599fd7062af72bea99438dc5d8c3aa66ab52ed7dee3e066c4e762bd4e42b0001599dd114ec6c4c5774929a342d530bf109b131b48db2d20855afa9d37c92d6390000019159393c84b1bf439d142ed2c54ee8d5f7599a8b8f95e4035a75c30b0ec0fa4c0128e3a018bd08b2a98ed8b6995826f5857a9dc2777ce6af86db1ae68b01c3c53d0000000001e3ec5d790cc9acc2586fc6e9ce5aae5f5aba32d33e386165c248c4a03ec8ed670000011f8322ef806eb2430dc4a7a41c1b344bea5be946efc7b4349c1c9edb14ff9d39"
        )
    ];

    find_checkpoint(height, checkpoints)
}

pub fn get_all_main_checkpoints() -> Vec<(u64, &'static str, &'static str)> {
    vec![
        (610000, "000000000218882f481e3b49ca3df819734b8d74aac91f69e848d7499b34b472",
                 "0192943f1eca6525cea7ea8e26b37c792593ed50cfe2be7a1ff551a08dc64b812f001000000001deef7ae5162a9942b4b9aa797137c5bdf60750e9548664127df99d1981dda66901747ad24d5daf294ce2a27aba923e16e52e7348eea3048c5b5654b99ab0a371200149d8aff830305beb3887529f6deb150ab012916c3ce88a6b47b78228f8bfeb3f01ff84a89890cfae65e0852bc44d9aa82be2c5d204f5aebf681c9e966aa46f540e000001d58f1dfaa9db0996996129f8c474acb813bfed452d347fb17ebac2e775e209120000000001319312241b0031e3a255b0d708750b4cb3f3fe79e3503fe488cc8db1dd00753801754bb593ea42d231a7ddf367640f09bbf59dc00f2c1d2003cc340e0c016b5b13"
        ),
        (630000, "00000000015493abba3e3bb384562f09141548f60581e06d4056993388d2ea2f",
                 "019b01066bae720ce88b4252c3852b0160ec4c4dcd6110df92e76de5cb23ab2f540109c3001b823fc745328a89a47fc5ace701bbd4dc1e9692e918a125ca48960545100001b2ba91c0f96777e735ded1ba9671003a399d435db3a0746bef3b2c83ba4d953f01d4c31130d2013fb57440d21fba0a8af65e61cd1405a8e2d9b987c02df8fc6514011c44ba36710e293ddf95e6715594daa927883d48cda6a3a5ee4aa3ef141ec55b0001cd9540592d39094703664771e61ce69d5b08539812886e0b9df509c80f938f6601178b3d8f9e7f7af7a1f4a049289195001abd96bb41e15b4010cecc1468af4e4b01ffe988e63aba31819640175d3fbb8c91b3c42d2f5074b4c075411d3a5c28e62801cb2e8d7f7387a9d31ba38697a9564808c9aff7d018a4cbdcd1c635edc3ab3014000001060f0c26ee205d7344bda85024a9f9a3c3022d52ea30dfb6770f4acbe168406d0103a7a58b1d7caef1531d521cc85de6fcb18d3590f31ad4486ca1252dac2c96020001319312241b0031e3a255b0d708750b4cb3f3fe79e3503fe488cc8db1dd00753801754bb593ea42d231a7ddf367640f09bbf59dc00f2c1d2003cc340e0c016b5b13"
        ),
        (650000, "0000000000a0a3fbbd739fb4fcbbfefff44efffc2064ca69a59d5284a2da26e2",
                 "01a6224d30bd854bb14e06b650e887e9ee3a45067dde6af8fdbca004b416accf0b001000018363c4cef8b386c64e759aba8380e950cae17e839da07426966b74ba23b06c350001ba6759797b2db9fbb295a6443f66e85a8f7b2f5895a6b5f5c328858e0af3bd4e00013617c00a1e03fb16a22189949e4888d3f105d10d9a7fcc0542d7ff62d9883e490000000000000163ab01f46a3bb6ea46f5a19d5bdd59eb3f81e19cfa6d10ab0fd5566c7a16992601fa6980c053d84f809b6abcf35690f03a11f87b28e3240828e32e3f57af41e54e01319312241b0031e3a255b0d708750b4cb3f3fe79e3503fe488cc8db1dd00753801754bb593ea42d231a7ddf367640f09bbf59dc00f2c1d2003cc340e0c016b5b13"
        ),
        (690000, "0000000000b1e6422ecd9292951b36ebb94e8926bbd33df8445b574b4be14f79",
                 "0117ffc074ef0f54651b2bc78d594e5ff786d9828ae78b1db972cd479669e8dd2401cc1b37d13f3b7d1fa2ead08493d275bfca976dd482e8dd879bf62b987652f63811013d84614158c7810753cc663f7a3da757f84f77744a24490eb07ce07af1daa92e0000017472a22c4064648ff260cbec8d85c273c5cd190dab7800f4978d473322dab1200001c7a1fd3786de051015c90f39143f3cfb89f2ea8bb5155520547ecfbefcdc382a0000000001d0c515cd513b49e397bf96d895a941aed4869ff2ff925939a34572c078dc16470121c1efd29f85680334050ee2a7e0d09fde474f90e573d85b7c9d337a5465625a0000000001d2ea556f49fb934dc76f087935a5c07788000b4e3aae24883adfec51b5f4d260"
        ),
        (750000, "00000000028522f87172ecefd79b5f54547c8a756976585f29e4dc182a19c46a",
                 "01a069618d376feebdbf39030c254a1a3cb46d19369837e44b6ad9afb43763167300110000010c256f47b493d8d94dd5ad09a6829a0a5e346400430b222072583afad8ce847101b261be33d5db156d09fa73031e8f37b4fe4193d21c909e2c8e58d86c7e48690a016b4a7608e90189275f7bb8e70f525c333431ceaa8de9d5b119e66ce2faeb79290000017d730339d1d4bf490eda3c1fca77d7b8a769fff083318ec46a81404fef45f046013ad81619e96171627f27cd6e7755c4d8261dc7017a65753f06c6cf9a29af116201474991dfe7d598257dae28820c6058e389a897e232e737c90a5427e8f24e355e0163734115d47b641de26abf2cad5c4ac1cb438869fc91d50e66444980647aed24000000017d066851cc49b2ea0cf9fb6af00adbb1cc3a0b15cb02d39e0a66f031b2dc1f230001d2ea556f49fb934dc76f087935a5c07788000b4e3aae24883adfec51b5f4d260"
        ),
        (780000, "00000000010b38c91627aeb8aadf44694220c904f443ddbbd8a4c6b71670b06c",
                 "01e2b62355ee2ff200c4fbee907ed9f1a35d0c79c4350b81d3f0a4326715022b0801a2a51439699ad5b3fc4b05497fa6563ea7302a674970e97bc5367c4d677f7b4411000000000196e5f4889d5487cf26623b27b5cb0cc9f07cb07bff97f0acf8e95dd4d70da63f0116b26415fa86d2ca9954fd959dec4a45617e6a3eb8bf66b7c817a508793eef1401f691350cf164f3f31e8857af98f612c2d6145bb375f11451b50a8de9b4f54f4e01ff661d9b20b556f96b0d3396bca15f6aa56e7e7b64b871d8c632538cb8980b44000106cd1b467e72f8fe72ce6d2d4e07180636ae582368d02c3d3565ec96f50d3f210172c15bb2d34dd3408b34b3ca6c7ab86f886cf64e4bfacf1435601c7526302b2a0183d162c2033fa5f8848429e2c38b6cca8333a41971178e00ce3abf663d8c496c01e07372caea187301a24f9dbbe3b6a8981bb4225b7f4b362b01000d4b0a0eb071011545fef7ddad5a52664dff5a73fbbb26c2cdf42aec9292349773079432d5bc46017d066851cc49b2ea0cf9fb6af00adbb1cc3a0b15cb02d39e0a66f031b2dc1f230001d2ea556f49fb934dc76f087935a5c07788000b4e3aae24883adfec51b5f4d260"
        ),
        (810000, "000000000288d460756352efd73bdffd34a686c2a6de2c75fc4ced2ede108fcd",
                 "01f3c97857d707d1b4be8bf323061e6c2e901dd8a9f75731c88e2d0c326a94262e014478d25f683bacae03664c72840cdd2c89eeb2206d3e23c48ad538bdeb2ea7541101a4641a722bf5b845e42ad03a80ec75b74a0bf5c8f44ab4ccec240aa05e6c5b120001c1bc6535533a3d698855a3946cf962bfd74f41aab868e4f450882afe9ff5df5500013a89db2ae718d644a2724a74b65f6f6af59f71528f33c65d67b39a76096cb82c012f6e7ac1ccbc575a9b16cd98fdc214e80b88905eb65aee511635b76499c1ca380155a9f8a53376407d03e995d28264924cffedca826a8eb508845c520dee82ab0600018d702b85682815a8a0c7da62a221148c03b1a77e64d773d1af5ba458e1b0b22d0001adb716a07c0268781317d5bf4d3ed65e5838f3111e86d937049eccccdee8f83a01f8f596c518693801a918f1ed44db811bd345a14e9cc2a038b6164cbcc679ca4301d278b23e92a9b00400f94eb0147f215acf22cd1f24a0b61329ca186cb4917b14014da3714363acb83872f51c87fed3d42a1093420c3cb96b74ad65966ce27e8c4e0001e2bf698f5ac10b44da560d11a5e1d5c191a82a968a2be0a6948aa8748b54516001d2ea556f49fb934dc76f087935a5c07788000b4e3aae24883adfec51b5f4d260"
        ),
        (840000, "00000000000a0b9a8753dfd46e1205590d35f4d365437a0d20d29317b33743c0",
                 "01101f7b8112735869abc0e74dac1f272935d09725ff03cd8cb63d472e112fa82a01d635e219273ade2859554e5191898ce9a79cb80f62c272dede1c2a71e183e21e120000000000000000000000000000000000011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (870000, "0000000001097864030cac84b7bb93d12739ce9391612c66758c76e3243f0306",
                 "01302886fbfdb837d575fc8fc2d8a7f74cb62a19ca60d2651eb19c5e6f486a4e22014408f734c0d7c683f37021404694b91dba5a0831c19035041c6bea83889af76912013cfc980f1a52aa4f2eb962c0b7bbd89e1a7e1f00dd1c8a62c0e03f118b4eb65b01cfe618a71029cf0bc39e796eeedc70ff9402959487f5825b5b15ede34b36021401bd79f262082e6a1ebdf586cd9c5b4726afd2d85bfb951eef97fb90537de86723012bc7fba59d8ac7d6c7d8538446b2dfcde28ee3c26f445938c8966196da9d456e019a599970b9798b4dc0f0ee59dd273a70b209515d95c510117db7eecebb36e80301c70fe44f3142eb00cc4f28d13d42173ce9e7f316e987c8fc3a2e01ee3b71bd2400000108348cb2dfc1ff7af214ad16c6bdb1d49ace2176a6aacea4d6ddc9d3a7cb9334000000000001166bb2e71749ab956e549119ce9099df3dbb053409ff89d0d86a17d5b02d015d0000011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (910000, "0000000000f90b683c2a3fef74e21872247b19ad4558b36ca3eea9405538cd70",
                 "010ece000f48ffec735cf7756501aa52be1a7265a37b388b25966659eb8ae45a400110f701d1ea41c927f85f6564a512b295b36eeee186394a88bf00863a7900fa591201fe016c729c11a542f7b044299c8125892390d8f62fa853b53e1eb07e20fc64450001e1ba264bfa5f4a14498e5ac811acae9ebc9bdca547f6dd851dd3a2eefaab0c2c0001bec5ba9b735a95467fa0e1fafd2090b29765f323deb07e79757722e5cd77b835017217b253186b5fb1b0d55959b7e77e520c29363b6ba8a9f73932fa42fa09c3530001e63960c1f08d2cc7bc0f141dbee83516a13fd52252192047936aff6ba1cf4d620130978181e2608adad8aefcf44a3bf56387265b35ccbd619c8930631c4364c03f00000133604c7020edaefee31d4a419a4067ccd09d369a43fe4c032eeb7081774ed53901390f92d515d7c479d7c9807949f237c50bc77a732f0853884f12d01b72a5e75401d3ddf137881180d7c5fd73d188c4346291168bde688643855eb3cd5f680f9c0001166bb2e71749ab956e549119ce9099df3dbb053409ff89d0d86a17d5b02d015d0000011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (960000, "0000000000b5b5e0ba1c01f76b8105878ea3c2f11da53cb0ec684f5d94365421",
                 "014695c74583a750216dbc0aec38282d86fc17b595bb45a74bbee8fdbf46b5313e01c2253474715b00c618e635815bd16fb8e6368fdaa9bf8f4a1aca34ead6a7eb1c12000000010cf46f452fc9101af9ca34ae364a1c2e20bc05d454068cf1407a2ee3e0c9ca6700000001091c0b4153defbfad723bf14e1ccd07c0258ea1fcd6e9e8cf759834112ec3036000001c2c980c0777874ce748ca549838324eb775cb2ac7a8d42793edbb05ac15c5b4201162d1417d8b9659ec93ac26ba1a888719a43ab1fe0b46a33c05c2aa55fecb41b00018e1d474609c9c09894638a0ab3e656aadccaf7ddf12bcc6b6ece44a4cc79e1140001f1c57245fff8dbc2d3efe5a0953eafdedeb06e18a3ad4f1e4042ee76623f803200011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (1000000, "000000000062eff9ae053020017bfef24e521a2704c5ec9ead2a4608ac70fc7a",
                 "01a4d1f92e2c051e039ca80b14a86d35c755d88ff9856a3c562da4ed14f77f5d0e0012000001f1ff712c8269b7eb11df56b23f8263d59bc4bb2bbc449973e1c85f399c433a0401e0e8b56e5d56de16c173d83c2d96d4e2f94ce0cbd323a53434c647deff020c08000129acf59ead19b76e487e47cf1d100e953acedc62afa6b384f91a620321b1585300018179961f79f609e6759032f3466067548244c3fe0bf31d275d1b6595bb2d486401b622d3f80d8231c44483faa2a27e96e06a2e08d099b26d15828e8f0bde0bd42001a8d1f585aeceb5c3f22ffb43014fe89db9f7efc080361d4fa4e8d596ab1224400103ee02ae59c6688dcaadf1c4ff95e7b1a902837e4989a4c4994dce7dac6ecb20014ff8c0fe6bce02ac4ad684996bfa931d61c724015d797642819361d611ebd61201c7ae83949d9502b0eff10618124d335f046e4aae52c19ccad5567feceb342a5200000001b7fc5791e3650729b7e1e38ee8c4ea9da612a07b4bf412cefaffbab7ac74c547011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (1030000, "000000000216b8552281f6b73332ec873e0eb062f9b83ede4f3102af78446b7c",
                "016f037481382b438c1a41d74153ee8a6131db5a9fcfa2526718e7b4fa1577e658001201622ea365daf3d6cbd5b712bb33b77764708d69f852b903f053c47569cabd930a0001e2175829e38f95f1b3415bdaa796a732d83da1b137e1b0aecfa9802b8c8e9a540001c783c98897bc46693d1d2d2891489663f0d9ff12b34f28b3db5a841236d9d76501d671aaba2921e416e7b7b81a89cb8e524cb6c49d89c575e04e0da0461799216f0001ba538b78350bfeae6538bfac75fe8709eb59bb72f6d74d64c92df41ac1e464560001ef2204037f952a1365afd291acf2361dcebda719b5e659de073ebe2f7f3eae1a01264c173c66e9b7c36ac9f7a6c928600107fa40f11b8d81862464f849885c50620189c3e3ed72b0d445f043e2d2d5ec23c693ef67b9a15488911ad35480a6041c6301f7f497a2f9ded8bb6d14d9f1bb83f43396270e1fc7e86c2c789d9b74b9d2d3070001bde7578541c07920c1bc94312596adfcee32519cb80a95bcd06a1272c127ff020001b7fc5791e3650729b7e1e38ee8c4ea9da612a07b4bf412cefaffbab7ac74c547011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (1080000, "0000000001a6faf5681b8565d50145fd84547b534c5f869e77cb802518d14341",
                "01f3955ce270f5718bf68883ed37b3b9d2de8fd77be7bd95334fbedc6083f16026001200000001bd5dd7584bc157cebc9d63c7ee761ab453892482246aae3ef9db17de80b84a4b000195fa995a764f9afbd6c14984dbc72175f49f2259bcf0abc4a82ac92446532c44000168fb4180546c77370ff4175d40a29c357e5787f820e383028243ba623fce4e61017cd28108a3c64a8923444af9b7409eb5dda47d8536cf5aafc80abf62e9551b3501fc0832fb90a473de0da1ae7f62b03d547655aa82d1f279c5ab5a997d6472085901647f2444d093ad8668eac738fe0ff6b59b8191bcbc13dc53f581e64de755122a000101e8d7f1b32b8bc1ec539b93f6c2912c839a55c36c509711340a5cf6d1803a360103bcde16c3ed62026afcdeb7c33c7aae0bbaaa357e8d67a10457244bdacabf4f0001891b1e6bfec42e97c79ec505c7ae1b584cf47d4ed8f6cdfcad815b02a5496f6701b7fc5791e3650729b7e1e38ee8c4ea9da612a07b4bf412cefaffbab7ac74c547011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625"
        ),
        (1140000, "00000000006d9c69d8c4d7818dd2b9b106078317c6e881eab86ba41e4a546337",
                "012afd078f200a60fe586c8ebb81208bc6b656f2c24935ed9ae483606368c6101c001301e0a84c415504116132c953c1885150a6335bc3293aa17c89626e02406f944f39000000017c0a218b969475ad665bfdea14177bd73b8510d451bd1f533f29d5f86f92a14201faee45bdbbec94f64fd8b641b3e2c1c473880b14a86134d766b9ffae127f0506014815011726d11513734103e47971902e4e8c1245ab2b96107184f2978a13cb2501eecd48ee70785ed9d31c4edb6da31287fe1b42082466c787262133017fe3ab210183352b54c9e84bed4c1fb4c31dc1bf690e88aec9f477f6e00d51a4bc918cba32018b702b3a1bb47c1455d5ca00cdb6d2eb72256f421937dee7d5453e7108c8df3a00017c41a5948f315986b60909e07a15c2639fb1b13a968aaf322d654aa1e823f60b00000116316e325ad5299b583121e42838f9cb432a88a7042cfeca8d36f8f5e86e234f0000000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1190000, "00000000019caeb64ab8caa12e1dcdd9a8f391b063ec252887c91526b7ac5e0c",
                "017ae2fbedf5cad68ee80df0ae9caef9e0168914780bfd14eae016e2fb89068071001301c78a8f9bfddd9a1f0076048c48d1d58298ac6f4368472a39b0b240d540117c4301b58c9284084704437af784307ab0a334dc9c7aef588bf7d26491274d79c4471301b0af0fff110293b555f17d5ced0a598693ff4cde3e680d988c6ccf498846753e01bb9d0f21f621448c03ee49de4ef3bf0faa4844544f9668197ef5921164d2401601a15d695922c5441e80aa770861f91a97dd460561515d32c9d06bd3c6f98ce26f000000014a772e6ce520bcedf07793ded6148fd3415ecbdd89c3efe183b6048f1fb4791c0001e281c5ec71bc1a301ad0d285f6f1aa2552907645b02f9d43160f5354c2be7c63012b4d8d83df48c8b5c35752268eb71a428aa05103809a887fb36519dedbc8de27017b610946256293c523b36cf95ec60f2c346d866d98d1276bbaba22e46815516d000001089a1f9d50a037cc66aba4400b1703bcbb66f5f2993fd0dd3bb726e35940916700000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1220000, "0000000000751d7ffb3aac6f6a66ed01aa4e991a96162c1749dc556e50fe6df0",
                "01c3bbf34ce189a5285144c79cc37c4e76385b803947819ea8bc43b1e8eb2c020801861a52e2fb19b150b0b7c7d7f879bd1ac4fa3a11ac9763db4f837f4db048896413013ddab8738a0cc8e00e96c7487d21a9771a7986b7e1e5d5fb0348d46caa18360c01d6b0010e9664a01a47a8acc9bf073255f6fb62f617cb491b249671da85001862000000012fe6dc35a5d51af73b73854855f9861775296a9c56d6aa9455be2762a101d7390168ee29d7b083e5af0d1895b2832a4fc63a9c7b6cea37b75d17d28e6f5842ee0c0159781dcd759c87f8bc8622bc19f9a8732c06b52897bfb6e0ddcbadb111d6a95601036e008ad224efaa9833fa9e790192dad7aab0ba71bf872430f48ba6aa0d1d1b00000169bce4bc91631cf6158d978b5ce6ad03f3e4cc3317c28964b575ca736f8b4d68000001ece344ca21dbd3b681f167163d4792165efe8239390afc13378e50d044fee65a01089a1f9d50a037cc66aba4400b1703bcbb66f5f2993fd0dd3bb726e35940916700000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1240000, "0000000002473bf56688195f05b6f5acea0f99506fca40ae72f2ab8c1fd7390d",
                "017b7c743693588568510844e4bc181d12ab6433dced0542d149cbec2b96ba526500130001cb8eccc27eb266385f9e4b880ff337b2ebdf10238dac74414fde8937dfa2264b0001bb0cb6201a0d003f1e4df17cfd4815c382ced6bf038468873495ff5c9c25412501ba626167f23eb767c229a0498b37e8322326779a3a6454ebcefd639056b3e64400013ff350f9e880271ea2c713c509491605ea24589c369262c18c417fdf1027305e0000000001849b1538147707b1c880e7eee02f29ada52e8a5a6c3a9027087493f41bd8304a00018e7922ca798cd3e26d3369ca2425ec19baa7d79407a979ec1090ae48fdcd094a01ece344ca21dbd3b681f167163d4792165efe8239390afc13378e50d044fee65a01089a1f9d50a037cc66aba4400b1703bcbb66f5f2993fd0dd3bb726e35940916700000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1300000, "00000000027222bdbcf9c5f807f851f97312ac6e0dbbc2b93f2be21a69c59d44",
                "01f5a97e2679a2bb9103caf37b825f92fcd73fff836234844dfcf1815394522b2c01526587b9b9e8aeb0eb572d81fec1f5127b8278ba0f57e451bd6b796596940a2213000131c7ff90fafff6159b8fb6544a2bcbba6c102903158fce8f9a9d3c6654abb23300013555cb7f4f79badeaca9bf2dca5a8704f0929053d50e95c03002f9a4d5286c3a01ad3557e11c1607ec888dc84f5f8899c3c79fb1f50b613946452ec7dd5e53763c0001c4583f4482b949390dba355fc8fa63019c83acd644ddd633cb50211d236f870600000001088da0d78eefd0c222507927e403b972d0890d0c31e08b02268fbe39ac4a6e170001edf82d4e2b4893ea2028ca8c5149e50a4c358b856d73f2de2b9a22034fa78f22012ffde6dccbef68b60cd7b4e7a8fe7989f5954fa4bacad01b247d16b9bfa5084000000125911f4524469c00ccb1ba69e64f0ee7380c8d17bbfc76ecd238421b86eb6e09000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1335000, "00000000001d428474214f2844ac7adacab9c9b706f89ebb24e1e43189edff2d",
                "01105d94f868041b1680f862dad6211ab815a30c79a63b839c2b2043ce6530834801e53ee3fef11ddfaef984c8653dffa0354929b79aad7321b00c10cb3b60c8b7111301f5693ff9b17a8fc0b032c192841d1fc08b7ec9fe4fcc2b628a550434af70886a01838a7001b5ed5dcdec7bce1ea4250bbeebe8c22aa27fd69e7baf343458e95c7101030f11dfda75a9e4a63bab19fe3bf92c545a3f58a57ca41ae7609290dad01436018923004af490f5718e834215ef61f2f60aee24685c1c2cffb3c686dff57ab82501eb86680f83fa0f9c47da3875645344a2734d56edcf1d99747ecbf25ea0e86e22000001cf6872911593b4f1af2fd03dce8a48d434af849ad1bc872442e7881bbc04e8610168fbde909e21c25e1a686fac9982ee11fb0d05da3568579bfba8b71f7632d62700012965494015cdab2ce010c1ae4ea88306c286128275de391dcf57d3fa85be7e1b01a090ee174239a34a5d684425d09006d238c6075a61c5842d0fc26043f09ccd7001a2b7ee187c7b8ce18ebda8600bed7695b12f7d35ac971ed6ee67184a7ceebd490001b35fe4a943a47404f68db220c77b0573e13c3378a65c6f2396f93be7609d8f2a000125911f4524469c00ccb1ba69e64f0ee7380c8d17bbfc76ecd238421b86eb6e09000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
        (1370000, "000000000266f9dfa1e64e21f9462ad6a039ab9fa7d088ff4dcef05f19ff6a0b",
                "010349f2d44dd5345afca2e43fcebea8830ba1dd5008c033d652e5cd1e183e5316001301a7c4738bc91c9f34e368d05651a6d7f5daf5055ffa69859bbb04911a54d66d19015880f32973a54a1ebabc2cacfe0b12fb6f1efe260ab560cf393b5fb7b1568a0a000001cfaeb4efed5a78e31ed76e78caa297010591797f34d614306805e12adbb1b402000000019408f50fbc9bd2582e9fe881b6b2142a65aa32d09109640a312cc5f0fd9f6560000001142382a32d4e6140bed51bc21866afb918df31853d4afd24df0e8fe88d98180b00000001cdd92a1e884cf1914dae8345423203832ec7bbf9d95ae50b82e4327b39d6d9120125911f4524469c00ccb1ba69e64f0ee7380c8d17bbfc76ecd238421b86eb6e09000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644"
        ),
    ]
}

fn get_main_checkpoint(height: u64) -> Option<(u64, &'static str, &'static str)> {
    find_checkpoint(height, get_all_main_checkpoints())
}

fn find_checkpoint(
    height: u64,
    chkpts: Vec<(u64, &'static str, &'static str)>,
) -> Option<(u64, &'static str, &'static str)> {
    // Find the closest checkpoint
    let mut heights = chkpts
        .iter()
        .map(|(h, _, _)| *h)
        .collect::<Vec<_>>();
    heights.sort();

    match get_first_lower_than(height, heights) {
        Some(closest_height) => chkpts
            .iter()
            .find(|(h, _, _)| *h == closest_height)
            .copied(),
        None => None,
    }
}

fn get_first_lower_than(
    height: u64,
    heights: Vec<u64>,
) -> Option<u64> {
    // If it's before the first checkpoint, return None.
    if heights.is_empty() || height < heights[0] {
        return None;
    }

    for (i, h) in heights.iter().enumerate() {
        if height < *h {
            return Some(heights[i - 1]);
        }
    }

    return Some(*heights.last().unwrap());
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_lower_than() {
        assert_eq!(get_first_lower_than(9, vec![10, 30, 40]), None);
        assert_eq!(get_first_lower_than(10, vec![10, 30, 40]).unwrap(), 10);
        assert_eq!(get_first_lower_than(11, vec![10, 30, 40]).unwrap(), 10);
        assert_eq!(get_first_lower_than(29, vec![10, 30, 40]).unwrap(), 10);
        assert_eq!(get_first_lower_than(30, vec![10, 30, 40]).unwrap(), 30);
        assert_eq!(get_first_lower_than(40, vec![10, 30, 40]).unwrap(), 40);
        assert_eq!(get_first_lower_than(41, vec![10, 30, 40]).unwrap(), 40);
        assert_eq!(get_first_lower_than(99, vec![10, 30, 40]).unwrap(), 40);
    }

    #[test]
    fn test_checkpoints() {
        assert_eq!(get_test_checkpoint(500000), None);
        assert_eq!(get_test_checkpoint(600000).unwrap().0, 600000);
        assert_eq!(get_test_checkpoint(625000).unwrap().0, 600000);
        assert_eq!(get_test_checkpoint(650000).unwrap().0, 650000);
        assert_eq!(get_test_checkpoint(655000).unwrap().0, 650000);

        assert_eq!(get_main_checkpoint(500000), None);
        assert_eq!(get_main_checkpoint(610000).unwrap().0, 610000);
        assert_eq!(get_main_checkpoint(625000).unwrap().0, 610000);
    }
}
