import {
  ensureAddress,
  getKeyring,
  Global_Api,
  Global_ModuleMetadata,
  hexToUtf8,
  initApi,
  NativeCurrencyID,
  unit,
  waitTx,
} from "./utils.mjs";
import {Keyring} from "@polkadot/api";
import {bnToBn} from "@polkadot/util";
import {Command} from "commander";

function perU16ToFloat(x) {
  return Number(x) / 65535.0;
}

function float2PerU16(x) {
  return Math.trunc(x * 65535.0);
}

async function proxyDeposit(num_proxies) {
  try {
    let deposit = await Global_Api.ws.call('nftmart_addClassAdminDeposit', [num_proxies], 10000);
    return bnToBn(deposit);
  } catch (e) {
    console.log(e);
    return null;
  }
}

async function nftDeposit(metadata) {
  try {
    let depositAll = await Global_Api.ws.call('nftmart_mintTokenDeposit', [metadata.length], 10000);
    return bnToBn(depositAll);
  } catch (e) {
    console.log(e);
    return null;
  }
}

async function classDeposit(metadata, name, description) {
  try {
    let [_deposit, depositAll] = await Global_Api.ws.call('nftmart_createClassDeposit', [metadata.length, name.length, description.length], 10000);
    return bnToBn(depositAll);
  } catch (e) {
    console.log(e);
    return null;
  }
}

async function getAuctionDeadline(allowDelay, deadline, lastBidBlock) {
  try {
    let d = await Global_Api.ws.call('nftmart_getAuctionDeadline', [allowDelay, deadline, lastBidBlock], 10000);
    return bnToBn(d);
  } catch (e) {
    console.log(e);
    return null;
  }
}

async function getDutchAuctionCurrentPrice(maxPrice, minPrice, createdBlock, deadline, currentBlock) {
  // console.log(maxPrice, minPrice, createdBlock, deadline, currentBlock);
  try {
    let price = await Global_Api.ws.call('nftmart_getDutchAuctionCurrentPrice', [maxPrice, minPrice, createdBlock, deadline, currentBlock], 10000);
    return bnToBn(price);
  } catch (e) {
    console.log(e);
    return null;
  }
}

function print_nft(classID, tokenID, nft, accountToken) {
  if (nft.isSome) {
    nft = nft.unwrap();
    nft = nft.toJSON();
    nft.metadata = hexToUtf8(nft.metadata.slice(2));
    try {
      nft.metadata = JSON.parse(nft.metadata);
    } catch (_e) {
    }
    nft.data.royalty_rate = `${perU16ToFloat(nft.data.royalty_rate) * 100}%`;
    if (!!accountToken) {
      console.log(`classID ${classID} tokenID ${tokenID} accountToken ${accountToken} tokenInfo ${JSON.stringify(nft)}`);
    } else {
      console.log(`classID ${classID} tokenID ${tokenID} tokenInfo ${JSON.stringify(nft)}`);
    }
  }
}

async function display_nft_by(classID, tokenID) {
  let nft = await Global_Api.query.ormlNft.tokens(classID, tokenID);
  print_nft(classID, tokenID, nft);
}

async function transfer(keyring, from, to, amount) {
  console.log("============== transfer ==============");
  from = keyring.addFromUri(from);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await Global_Api.tx.balances.transfer(ensureAddress(keyring, to), bnToBn(amount).mul(unit)).signAndSend(from, a);
  await b();
}

async function main() {
  const ss58Format = 50;
  const keyring = new Keyring({type: 'sr25519', ss58Format});
  const program = new Command();
  program.option('--ws <url>', 'node ws addr', 'ws://192.168.0.5:9944');

  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' make_data
  // node nft-apis.mjs make_data
  program.command('make_data').action(async () => {
    const ws = program.opts().ws;
    const sudo = '//Alice';
    await add_whitelist(ws, keyring, sudo, "//Alice2");
    await update_auction_close_delay(ws, 3);

    const classId = 56;

    await create_class(ws, keyring, "//Alice");
    await mint_nft(ws, keyring, "//Alice", classId, 20, 0.14);
    await mint_nft(ws, keyring, "//Alice", classId, 21, 0.17);
    await mint_nft(ws, keyring, "//Alice", classId, 22, 0.11);
    await transfer_nfts(ws, keyring, [[classId, 0, 2], [classId, 1, 2], [classId, 2, 2]], "//Alice", "//Alice2");

    await add_class_admin(ws, keyring, "//Alice", classId, "//Bob");

    await create_class(ws, keyring, "//Bob");
    await mint_nft_by_proxy(ws, keyring, "//Bob", classId + 1, 20, 0.19);
    await burn_nft(ws, keyring, "//Bob", classId + 1, 0, 20);
    await destroy_class(ws, keyring, "//Bob", classId + 1);

    await create_category(ws, keyring, "//Alice", "my category");
    await submit_order(ws, keyring, "//Alice", [[classId, 0, 2], [classId, 1, 2], [classId, 2, 2]]);
    await submit_order(ws, keyring, "//Alice", [[classId, 0, 3], [classId, 1, 3], [classId, 2, 3]]);
    await submit_offer(ws, keyring, "//Bob", [[classId, 0, 2], [classId, 1, 2], [classId, 2, 2]]);
    await submit_offer(ws, keyring, "//Bob", [[classId, 0, 3], [classId, 1, 3], [classId, 2, 3]]);

    let orderIds = await show_order(ws, keyring, false);
    let offerIds = await show_offer(ws, keyring, false);

    await transfer(keyring, "//Alice", "//Alice2", "100000");

    await take_order(ws, keyring, "//Alice2", orderIds[0], "//Alice");
    await take_offer(ws, keyring, "//Alice2", offerIds[0], "//Bob");

    await remove_order(ws, keyring, "//Alice", orderIds[1]);
    await remove_offer(ws, keyring, "//Bob", offerIds[1]);

    orderIds = await show_order(ws, keyring, false);
    offerIds = await show_offer(ws, keyring, false);
    console.log("orderIds", orderIds, "offerIds", offerIds);

    await submit_dutch_auction(ws, "//Alice", true, 10, [[classId, 0, 1], [classId, 1, 2], [classId, 2, 3]]);
    await submit_dutch_auction(ws, "//Alice", false, 10, [[classId, 0, 1], [classId, 1, 2], [classId, 2, 3]]);
    await submit_british_auction(ws, "//Alice", true, 10, [[classId, 0, 1], [classId, 1, 2], [classId, 2, 3]]);
    await submit_british_auction(ws, "//Alice", false, 10, [[classId, 0, 1], [classId, 1, 2], [classId, 2, 3]]);

    await remove_dutch_auction(ws, "//Alice", 5);
    await remove_british_auction(ws, "//Alice", 7);
  });

  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' create_class //Alice
  program.command('create_class <signer>').action(async (signer) => {
    await create_class(program.opts().ws, keyring, signer);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_class_admin //Alice 0 //Bob
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_class_admin //Alice 0 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw
  program.command('add_class_admin <admin> <classId> <newAdmin>').action(async (admin, classId, newAdmin) => {
    await add_class_admin(program.opts().ws, keyring, admin, classId, newAdmin);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_class
  program.command('show_class').action(async () => {
    await show_class(program.opts().ws);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_whitelist
  program.command('show_whitelist').action(async () => {
    await show_whitelist(program.opts().ws, keyring);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_whitelist //Alice //Bob
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_whitelist //Alice 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw
  program.command('add_whitelist <sudo> <account>').action(async (sudo, account) => {
    await add_whitelist(program.opts().ws, keyring, sudo, account);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' mint_nft //Alice 0 30
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' mint_nft //Alice 0 30 true
  // 3. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' mint_nft //Alice 0 30 false
  // 4. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' mint_nft //Alice 0 30 false proxy
  program.command('mint_nft <admin> <classID> <quantity> [royalty_rate] [proxy]').action(async (admin, classID, quantity, royalty_rate, proxy) => {
    if (royalty_rate === undefined || royalty_rate === 'null') {
      royalty_rate = null;
    } else {
      royalty_rate = float2PerU16(Number(royalty_rate));
    }
    if (!!proxy) {
      await mint_nft_by_proxy(program.opts().ws, keyring, admin, classID, quantity, royalty_rate);
    } else {
      await mint_nft(program.opts().ws, keyring, admin, classID, quantity, royalty_rate);
    }
  });
  // 1: node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nft_by_class
  // 2: node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nft_by_class 0
  program.command('show_nft_by_class [classID]').action(async (classID) => {
    await show_nft_by_class(program.opts().ws, classID);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nft_by_account //Alice
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nft_by_account 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB
  program.command('show_nft_by_account <account>').action(async (account) => {
    await show_nft_by_account(program.opts().ws, keyring, account);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_account_by_nft 0 0
  program.command('show_account_by_nft <classId> <tokenId>').action(async (classId, tokenId) => {
    await show_account_by_nft(program.opts().ws, keyring, classId, tokenId);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_class_by_account //Alice
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_class_by_account 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB
  program.command('show_class_by_account <account>').action(async (account) => {
    await show_class_by_account(program.opts().ws, keyring, account);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' transfer_nfts //Alice 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB \
  //		--classId 0 --tokenId 0 --quantity 1 \
  //		--classId 0 --tokenId 1 --quantity 2 \
  //		--classId 0 --tokenId 2 --quantity 3
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' transfer_nfts //Alice //Bob \
  //		--classId 0 --tokenId 0 --quantity 1 \
  //		--classId 0 --tokenId 1 --quantity 2 \
  //		--classId 0 --tokenId 2 --quantity 3
  program.command('transfer_nfts <from> <to>')
    .requiredOption('-c, --classId <classIds...>')
    .requiredOption('-t, --tokenId <tokenIds...>')
    .requiredOption('-q, --quantity <quantities...>')
    .action(async (from, to, {classId, tokenId, quantity}) => {
      if (classId.length === tokenId.length && tokenId.length === quantity.length) {
        const tokens = classId.map((e, i) => {
          return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
        });
        await transfer_nfts(program.opts().ws, keyring, tokens, from, to);
      } else {
        console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
      }
    });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' burn_nft //Alice 0 0 20
  program.command('burn_nft <signer> <classID> <tokenID> <quantity>').action(async (signer, classID, tokenID, quantity) => {
    await burn_nft(program.opts().ws, keyring, signer, classID, tokenID, quantity);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' destroy_class //Alice 0
  program.command('destroy_class <signer> <classID>').action(async (signer, classID) => {
    await destroy_class(program.opts().ws, keyring, signer, classID);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' create_category //Alice 'my cate'
  program.command('create_category <signer> <metadata>').action(async (signer, metadata) => {
    await create_category(program.opts().ws, keyring, signer, metadata);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_category
  program.command('show_category').action(async () => {
    await show_category(program.opts().ws);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' submit_order //Alice
  //		--classId 0 --tokenId 0 --quantity 1 \
  //		--classId 0 --tokenId 1 --quantity 2 \
  //		--classId 0 --tokenId 2 --quantity 3
  program.command('submit_order <account>')
    .requiredOption('--classId <classIds...>')
    .requiredOption('--tokenId <tokenIds...>')
    .requiredOption('--quantity <quantities...>')
    .action(async (account, {classId, tokenId, quantity}) => {
      if (classId.length === tokenId.length && tokenId.length === quantity.length) {
        const tokens = classId.map((e, i) => {
          return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
        });
        await submit_order(program.opts().ws, keyring, account, tokens);
      } else {
        console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
      }
    });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_order
  program.command('show_order').action(async () => {
    await show_order(program.opts().ws, keyring, true);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' take_order //Bob 1 //Alice
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' take_order //Bob 1 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB
  program.command('take_order <account> <orderId> <orderOwner>').action(async (account, orderId, orderOwner) => {
    await take_order(program.opts().ws, keyring, account, orderId, orderOwner);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' remove_order //Alice 1
  program.command('remove_order <account> <orderId>').action(async (account, orderId) => {
    await remove_order(program.opts().ws, keyring, account, orderId);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' submit_offer //Alice
  //		--classId 0 --tokenId 0 --quantity 1 \
  //		--classId 0 --tokenId 1 --quantity 2 \
  //		--classId 0 --tokenId 2 --quantity 3
  program.command('submit_offer <account>')
    .requiredOption('--classId <classIds...>')
    .requiredOption('--tokenId <tokenIds...>')
    .requiredOption('--quantity <quantities...>')
    .action(async (account, {classId, tokenId, quantity}) => {
      if (classId.length === tokenId.length && tokenId.length === quantity.length) {
        const tokens = classId.map((e, i) => {
          return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
        });
        await submit_offer(program.opts().ws, keyring, account, tokens);
      } else {
        console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
      }
    });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_offer
  program.command('show_offer').action(async () => {
    await show_offer(program.opts().ws, keyring, true);
  });
  // 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' take_offer //Alice 1 //Bob
  // 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' take_offer //Alice 1 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw
  program.command('take_offer <account> <offerId> <offerOwner>').action(async (account, offerId, offerOwner) => {
    await take_offer(program.opts().ws, keyring, account, offerId, offerOwner);
  });
  // node nft-apis.mjs --ws 'ws://81.70.132.13:9944' remove_offer //Alice 1
  program.command('remove_offer <account> <offerId>').action(async (account, offerId) => {
    await remove_offer(program.opts().ws, keyring, account, offerId);
  });
  /*
    node nft-apis.mjs submit_dutch_auction //Alice true 120 \
      --classId 1 --tokenId 0 --quantity 1 \
      --classId 1 --tokenId 1 --quantity 2 \
      --classId 1 --tokenId 2 --quantity 3
    node nft-apis.mjs submit_dutch_auction //Alice false 120 \
      --classId 1 --tokenId 0 --quantity 1 \
      --classId 1 --tokenId 1 --quantity 2 \
      --classId 1 --tokenId 2 --quantity 3
  */
  program.command('submit_dutch_auction <account> <allow_british_auction> <deadline_minute>')
    .requiredOption('--classId <classIds...>')
    .requiredOption('--tokenId <tokenIds...>')
    .requiredOption('--quantity <quantities...>')
    .action(async (account, allow_british_auction, deadline_minute, {classId, tokenId, quantity}) => {
      if (classId.length === tokenId.length && tokenId.length === quantity.length) {
        const tokens = classId.map((e, i) => {
          return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
        });
        allow_british_auction = allow_british_auction !== 'false';
        await submit_dutch_auction(program.opts().ws, account, allow_british_auction, deadline_minute, tokens);
      } else {
        console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
      }
    });
  // node nft-apis.mjs show_dutch_auction
  program.command('show_dutch_auction').action(async () => {
    await show_dutch_auction(program.opts().ws);
  });
  // node nft-apis.mjs bid_dutch_auction //Bob //Alice 5 33
  program.command('bid_dutch_auction <bidder> <auctionCreatorAddress> <auctionId> <price>')
    .action(async (bidder, auctionCreatorAddress, auctionId, price) => {
      await bid_dutch_auction(program.opts().ws, bidder, auctionCreatorAddress, auctionId, price);
    });
  // node nft-apis.mjs redeem_dutch_auction //Bob //Alice 5
  program.command('redeem_dutch_auction <bidder> <auctionCreatorAddress> <auctionId>')
    .action(async (bidder, auctionCreatorAddress, auctionId) => {
      await redeem_dutch_auction(program.opts().ws, bidder, auctionCreatorAddress, auctionId);
    });
  // node nft-apis.mjs remove_dutch_auction //Alice 5
  program.command('remove_dutch_auction <auctionCreatorAddress> <auctionId>')
    .action(async (auctionCreatorAddress, auctionId) => {
      await remove_dutch_auction(program.opts().ws, auctionCreatorAddress, auctionId);
    });
  /*
    node nft-apis.mjs submit_british_auction //Alice true 10 \
      --classId 1 --tokenId 0 --quantity 1 \
      --classId 1 --tokenId 1 --quantity 2 \
      --classId 1 --tokenId 2 --quantity 3
  */
  program.command('submit_british_auction <account> <allow_delay> <deadline_minute>')
    .requiredOption('--classId <classIds...>')
    .requiredOption('--tokenId <tokenIds...>')
    .requiredOption('--quantity <quantities...>')
    .action(async (account, allow_delay, deadline_minute, {classId, tokenId, quantity}) => {
      if (classId.length === tokenId.length && tokenId.length === quantity.length) {
        const tokens = classId.map((e, i) => {
          return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
        });
        allow_delay = allow_delay !== 'false';
        await submit_british_auction(program.opts().ws, account, allow_delay, deadline_minute, tokens);
      } else {
        console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
      }
    });
  // node nft-apis.mjs show_british_auction
  program.command('show_british_auction').action(async () => {
    await show_british_auction(program.opts().ws);
  });
  // node nft-apis.mjs bid_british_auction //Bob //Alice 8 33
  program.command('bid_british_auction <bidder> <auctionCreatorAddress> <auctionId> <price>')
    .action(async (bidder, auctionCreatorAddress, auctionId, price) => {
      await bid_british_auction(program.opts().ws, bidder, auctionCreatorAddress, auctionId, price);
    });
  // node nft-apis.mjs redeem_british_auction //Bob //Alice 8
  program.command('redeem_british_auction <bidder> <auctionCreatorAddress> <auctionId>')
    .action(async (bidder, auctionCreatorAddress, auctionId) => {
      await redeem_british_auction(program.opts().ws, bidder, auctionCreatorAddress, auctionId);
    });
  // node nft-apis.mjs remove_british_auction //Alice 7
  program.command('remove_british_auction <auctionCreatorAddress> <auctionId>')
    .action(async (auctionCreatorAddress, auctionId) => {
      await remove_british_auction(program.opts().ws, auctionCreatorAddress, auctionId);
    });
  await program.parseAsync(process.argv);
}

async function remove_british_auction(ws, auctionCreatorAddress, auctionId) {
  console.log("============== remove_british_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  auctionCreatorAddress = keyring.addFromUri(auctionCreatorAddress);
  const call = Global_Api.tx.nftmartAuction.removeBritishAuction(auctionId);
  const feeInfo = await call.paymentInfo(auctionCreatorAddress);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(auctionCreatorAddress, a);
  await b();
}

async function redeem_british_auction(ws, signer, auctionCreatorAddress, auctionId) {
  console.log("============== redeem_british_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  signer = keyring.addFromUri(signer);
  const call = Global_Api.tx.nftmartAuction.redeemBritishAuction(ensureAddress(keyring, auctionCreatorAddress), auctionId);
  const feeInfo = await call.paymentInfo(signer);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(signer, a);
  await b();
}

async function bid_british_auction(ws, bidder, auctionCreatorAddress, auctionId, price) {
  console.log("============== bid_british_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  auctionCreatorAddress = ensureAddress(keyring, auctionCreatorAddress);
  price = bnToBn(price);
  let call = Global_Api.tx.nftmartAuction.bidBritishAuction(price.mul(unit), auctionCreatorAddress, auctionId);
  bidder = keyring.addFromUri(bidder);
  const feeInfo = await call.paymentInfo(bidder);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(bidder, a);
  await b();
}

async function show_british_auction(ws) {
  await initApi(ws);
  const keyring = getKeyring();
  const currentBlock = Number((await Global_Api.rpc.chain.getBlock()).block.header.number);

  const auctions = await Global_Api.query.nftmartAuction.britishAuctions.entries();
  for (const auction of auctions) {
    let key = auction[0];
    const len = key.length;
    const auctionId = Buffer.from(key.buffer.slice(len - 8, len)).readBigUInt64LE();
    key = key.buffer.slice(len - 32 - 8 - 8, len - 8 - 8);
    const address = keyring.encodeAddress(new Uint8Array(key));
    let data = auction[1].toHuman();
    let jsonData = auction[1].toJSON();
    data.creator = address;
    data.currentBlock = currentBlock;
    data.minRaise = perU16ToFloat(jsonData.minRaise);
    data.auctionId = auctionId.toString();

    let bid = await Global_Api.query.nftmartAuction.britishAuctionBids(data.auctionId);
    if (bid.isSome) {
      bid = bid.unwrap();
      if (bid.lastBidAccount.isSome) {
        const actualDeadline = await getAuctionDeadline(jsonData.allowDelay, jsonData.deadline, bid.lastBidBlock);
        data.actualDeadline = actualDeadline.toString();
        data.lastBidAccount = bid.lastBidAccount.unwrap();
        data.lastBidBlock = bid.lastBidBlock;
        data.lastBidPrice = `${bid.lastBidPrice / unit} NMT`;
      } else {
        data.actualDeadline = jsonData.deadline;
        data.lastBidAccount = "";
        data.lastBidBlock = 0;
        data.lastBidPrice = 0;
      }
    }

    data.closed = currentBlock > Number(data.actualDeadline);
    console.log("%s", JSON.stringify(data));
  }
}

async function submit_british_auction(ws, account, allow_delay, deadline_minute, tokens) {
  console.log("============== submit_british_auction ==============");

  await initApi(ws);
  const keyring = getKeyring();
  let blockTimeSec = Global_Api.consts.babe.expectedBlockTime.toNumber() / 1000;
  let deadlineBlock = deadline_minute * 60 / blockTimeSec;
  let block = await Global_Api.rpc.chain.getBlock();
  deadlineBlock += Number(block.block.header.number);

  account = keyring.addFromUri(account);

  let min_deposit = (await Global_Api.query.nftmartConf.minOrderDeposit()).toString();
  const categoryId = 0;
  const init_price = 10 * unit;
  const hammer_price = bnToBn('10000').mul(unit);
  // `float2PerU16(0.5)`:
  //
  // For any bidding of an auction,
  // the second bidding should at least to be 1.5 times of the first relative biding.
  const minRaise = float2PerU16(0.5); // 50%
  const call = Global_Api.tx.nftmartAuction.submitBritishAuction(
    NativeCurrencyID, hammer_price, minRaise, min_deposit, init_price,
    deadlineBlock, allow_delay, categoryId, tokens);

  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function remove_dutch_auction(ws, auctionCreatorAddress, auctionId) {
  console.log("============== remove_dutch_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  auctionCreatorAddress = keyring.addFromUri(auctionCreatorAddress);
  const call = Global_Api.tx.nftmartAuction.removeDutchAuction(auctionId);
  const feeInfo = await call.paymentInfo(auctionCreatorAddress);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(auctionCreatorAddress, a);
  await b();
}

async function redeem_dutch_auction(ws, signer, auctionCreatorAddress, auctionId) {
  console.log("============== redeem_dutch_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  signer = keyring.addFromUri(signer);
  const call = Global_Api.tx.nftmartAuction.redeemDutchAuction(ensureAddress(keyring, auctionCreatorAddress), auctionId);
  const feeInfo = await call.paymentInfo(signer);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(signer, a);
  await b();
}

async function bid_dutch_auction(ws, bidder, auctionCreatorAddress, auctionId, price) {
  console.log("============== bid_dutch_auction ==============");
  await initApi(ws);
  const keyring = getKeyring();
  auctionCreatorAddress = ensureAddress(keyring, auctionCreatorAddress);
  let [auction, bid, block] = await Promise.all([
    Global_Api.query.nftmartAuction.dutchAuctions(auctionCreatorAddress, auctionId),
    Global_Api.query.nftmartAuction.dutchAuctionBids(auctionId),
    Global_Api.rpc.chain.getBlock(),
  ]);

  const currentBlock = Number(block.block.header.number);
  if (auction.isSome && bid.isSome) {
    auction = auction.unwrap();
    bid = bid.unwrap();
    let call;
    if (bid.lastBidAccount.isNone) {
      // This is the first bidding.
      if (currentBlock > auction.deadline) {
        console.log("auction closed");
        return;
      }
      const uselessPrice = 0; // The real price used will be calculated by Dutch auction logic.
      call = Global_Api.tx.nftmartAuction.bidDutchAuction(uselessPrice, auctionCreatorAddress, auctionId);
    } else {
      // This if branch is at least the second bidding.

      const deadline = await getAuctionDeadline(true, 0, bid.lastBidBlock);
      if (currentBlock > deadline) {
        console.log("auction closed");
        return;
      }

      const minRaise = perU16ToFloat(auction.minRaise);
      const lowest = (1 + minRaise) * (bid.lastBidPrice / unit);
      if (price > lowest) {
        call = Global_Api.tx.nftmartAuction.bidDutchAuction(price * unit, auctionCreatorAddress, auctionId);
      } else {
        console.log("price %s NMT should be greater than %s NMT", price, lowest);
        return;
      }
    }
    bidder = keyring.addFromUri(bidder);
    const feeInfo = await call.paymentInfo(bidder);
    console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
    let [a, b] = waitTx(Global_ModuleMetadata);
    await call.signAndSend(bidder, a);
    await b();
  } else {
    console.log("auction %s not found", auctionId);
  }
}

async function show_dutch_auction(ws) {
  await initApi(ws);
  const keyring = getKeyring();
  const currentBlock = Number((await Global_Api.rpc.chain.getBlock()).block.header.number);
  // For querying an auction. If you already have owner and auctionId, you can alternatively
  // use `Global_Api.query.nftmartAuction.dutchAuctions(auctionCreatorAddress, auctionId)`
  //
  // For iterating all auctions of an address:
  // `Global_Api.query.nftmartAuction.dutchAuctions.entries(auctionCreatorAddress)`
  //
  // For iterating all auctions on the nftmart blockchain:
  // `Global_Api.query.nftmartAuction.dutchAuctions.entries()`
  const auctions = await Global_Api.query.nftmartAuction.dutchAuctions.entries();
  for (const auction of auctions) {
    let key = auction[0];
    const len = key.length;
    const auctionId = Buffer.from(key.buffer.slice(len - 8, len)).readBigUInt64LE();
    key = key.buffer.slice(len - 32 - 8 - 8, len - 8 - 8);
    const address = keyring.encodeAddress(new Uint8Array(key));
    let data = auction[1].toHuman();
    let jsonData = auction[1].toJSON();
    data.creator = address;
    data.currentBlock = currentBlock;
    data.minRaise = perU16ToFloat(jsonData.minRaise);
    data.auctionId = auctionId.toString();

    {
      const currentPrice = await getDutchAuctionCurrentPrice(jsonData.maxPrice, jsonData.minPrice, jsonData.createdBlock, jsonData.deadline, currentBlock);
      data.currentPrice = `${currentPrice / unit} NMT`;
    }

    let bid = await Global_Api.query.nftmartAuction.dutchAuctionBids(data.auctionId);
    if (bid.isSome) {
      bid = bid.unwrap();
      if (bid.lastBidAccount.isSome) {
        const actualDeadline = await getAuctionDeadline(true, 0, bid.lastBidBlock);
        data.actualDeadline = actualDeadline.toString();
        data.lastBidAccount = bid.lastBidAccount.unwrap();
        data.lastBidBlock = bid.lastBidBlock;
        data.lastBidPrice = `${bid.lastBidPrice / unit} NMT`;
      } else {
        data.actualDeadline = jsonData.deadline;
        data.lastBidAccount = "";
        data.lastBidBlock = 0;
        data.lastBidPrice = 0;
      }
    }

    data.closed = currentBlock > Number(data.actualDeadline);
    console.log("%s", JSON.stringify(data));
  }
}

async function submit_dutch_auction(ws, account, allow_british_auction, deadline_minute, tokens) {
  console.log("============== submit_dutch_auction ==============");

  await initApi(ws);
  const keyring = getKeyring();
  let blockTimeSec = Global_Api.consts.babe.expectedBlockTime.toNumber() / 1000;
  let deadlineBlock = deadline_minute * 60 / blockTimeSec;
  let block = await Global_Api.rpc.chain.getBlock();
  deadlineBlock += Number(block.block.header.number);

  account = keyring.addFromUri(account);

  let min_deposit = (await Global_Api.query.nftmartConf.minOrderDeposit()).toString();
  const categoryId = 0;
  const min_price = 10 * unit;
  const max_price = 100 * unit;
  // `float2PerU16(0.5)`:
  //
  // For any bidding of an auction,
  // the second bidding should at least to be 1.5 times of the first relative biding.
  const minRaise = float2PerU16(0.5); // 50%
  const call = Global_Api.tx.nftmartAuction.submitDutchAuction(
    NativeCurrencyID, categoryId, min_deposit, min_price, max_price,
    deadlineBlock, tokens, allow_british_auction, minRaise);

  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function remove_offer(ws, keyring, account, offerId) {
  console.log("============== remove_offer ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);
  const call = Global_Api.tx.nftmartOrder.removeOffer(offerId);
  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function take_offer(ws, keyring, account, offerId, offerOwner) {
  console.log("============== take_offer ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);
  offerOwner = ensureAddress(keyring, offerOwner);
  const call = Global_Api.tx.nftmartOrder.takeOffer(offerId, offerOwner);
  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
  // console.log("assets of offer owner(%s):", offerOwner);
  // await show_nft_by_account(ws, keyring, offerOwner);
  // console.log("assets of signer(%s):", account.address);
  // await show_nft_by_account(ws, keyring, account.address);
}

async function show_offer(ws, keyring, show) {
  await initApi(ws);
  const currentBlockNumber = bnToBn(await Global_Api.query.system.number());
  let offerCount = 0;
  const allOffers = await Global_Api.query.nftmartOrder.offers.entries();
  let offerIds = [];
  for (let offer of allOffers) {
    let key = offer[0];
    let keyLen = key.length;

    const offerId = Buffer.from(key.buffer.slice(keyLen - 8, keyLen)).readBigUInt64LE();
    const offerOwner = keyring.encodeAddress(new Uint8Array(key.buffer.slice(keyLen - 8 - 8 - 32, keyLen - 8 - 8)));
    offerIds.push(offerId);

    let data = offer[1].toHuman();
    data.offerOwner = offerOwner;
    data.offerId = offerId.toString();

    if (show) {
      console.log("\n\noffer: %s", JSON.stringify(data));
      for (const item of data.items) {
        await display_nft_by(item.classId, item.tokenId);
      }
    }
    offerCount++;
  }
  if (show) {
    console.log(`offer count is ${offerCount}. current block is ${currentBlockNumber}`);
  }
  return offerIds;
}

async function submit_offer(ws, keyring, account, tokens) {
  console.log("============== submit_offer ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);

  const price = unit.mul(bnToBn('20'));
  const categoryId = 0;
  const currentBlockNumber = bnToBn(await Global_Api.query.system.number());

  const call = Global_Api.tx.nftmartOrder.submitOffer(
    NativeCurrencyID,
    categoryId,
    price,
    currentBlockNumber.add(bnToBn('300000')),
    tokens,
  );

  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function remove_order(ws, keyring, account, orderId) {
  console.log("============== remove_order ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);
  const call = Global_Api.tx.nftmartOrder.removeOrder(orderId);
  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function take_order(ws, keyring, account, orderId, orderOwner) {
  console.log("============== take_order ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);
  orderOwner = ensureAddress(keyring, orderOwner);
  const call = Global_Api.tx.nftmartOrder.takeOrder(orderId, orderOwner);
  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
  // console.log("assets of order owner(%s):", orderOwner);
  // await show_nft_by_account(ws, keyring, orderOwner);
  // console.log("assets of signer(%s):", account.address);
  // await show_nft_by_account(ws, keyring, account.address);
}

async function show_order(ws, keyring, show) {
  await initApi(ws);
  const currentBlockNumber = bnToBn(await Global_Api.query.system.number());
  let orderCount = 0;
  const allOrders = await Global_Api.query.nftmartOrder.orders.entries();
  let orderIds = [];
  for (let order of allOrders) {
    let key = order[0];
    let keyLen = key.length;

    const orderId = Buffer.from(key.buffer.slice(keyLen - 8, keyLen)).readBigUInt64LE();
    const orderOwner = keyring.encodeAddress(new Uint8Array(key.buffer.slice(keyLen - 8 - 8 - 32, keyLen - 8 - 8)));
    orderIds.push(orderId);

    let data = order[1].toHuman();
    data.orderOwner = orderOwner;
    data.orderId = orderId.toString();

    if (show) {
      console.log("\n\norder %s", JSON.stringify(data));
      for (const item of data.items) {
        await display_nft_by(item.classId, item.tokenId);
      }
    }
    orderCount++;
  }
  if (show) {
    console.log(`order count is ${orderCount}. current block is ${currentBlockNumber}`);
  }
  return orderIds;
}

async function submit_order(ws, keyring, account, tokens) {
  console.log("============== submit_order ==============");

  await initApi(ws);
  account = keyring.addFromUri(account);

  const price = unit.mul(bnToBn('20'));
  const deposit = unit.mul(bnToBn('5'));
  const categoryId = 0;
  const currentBlockNumber = bnToBn(await Global_Api.query.system.number());

  const call = Global_Api.tx.nftmartOrder.submitOrder(
    NativeCurrencyID,
    categoryId,
    deposit,
    price,
    currentBlockNumber.add(bnToBn('300000')),
    tokens,
  );

  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s NMT", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(account, a);
  await b();
}

async function show_category(ws) {
  await initApi(ws);
  let cateCount = 0;
  const callCategories = await Global_Api.query.nftmartConf.categories.entries();
  let cateIds = [];
  for (let category of callCategories) {
    let key = category[0];
    const data = category[1].unwrap();
    const len = key.length;
    const cateId = Buffer.from(key.buffer.slice(len - 8, len)).readBigUInt64LE();
    cateIds.push(cateId);
    console.log(cateId.toString(), data.toHuman());
    cateCount++;
  }
  console.log(`cateCount is ${cateCount}.`);
  return cateIds;
}

async function create_category(ws, keyring, signer, metadata) {
  console.log("============== create_category ==============");

  await initApi(ws);
  signer = keyring.addFromUri(signer);
  const call = Global_Api.tx.sudo.sudo(Global_Api.tx.nftmartConf.createCategory(metadata));
  const feeInfo = await call.paymentInfo(signer);
  console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(signer, a);
  await b();
}

async function update_auction_close_delay(ws, minute) {
  console.log("============== update_auction_close_delay ==============");
  await initApi(ws);
  const keyring = getKeyring();
  const signer = keyring.addFromUri("//Alice");
  let blockTimeSec = Global_Api.consts.babe.expectedBlockTime.toNumber() / 1000;
  const call = Global_Api.tx.sudo.sudo(Global_Api.tx.nftmartConf.updateAuctionCloseDelay(Number(minute) * 60 / blockTimeSec));
  const feeInfo = await call.paymentInfo(signer);
  console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(signer, a);
  await b();
}

async function destroy_class(ws, keyring, signer, classID) {
  console.log("============== destroy_class ==============");

  await initApi(ws);
  // await show_class_by_account(ws, keyring, signer);
  const sk = keyring.addFromUri(signer);
  let classInfo = await Global_Api.query.ormlNft.classes(classID);
  if (classInfo.isSome) {
    classInfo = classInfo.unwrap();
    const call = Global_Api.tx.proxy.proxy(classInfo.owner, null, Global_Api.tx.nftmart.destroyClass(classID, sk.address));
    const feeInfo = await call.paymentInfo(sk);
    console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
    let [a, b] = waitTx(Global_ModuleMetadata);
    await call.signAndSend(sk, a);
    await b();
  }
  // await show_class_by_account(ws, keyring, signer);
}

async function burn_nft(ws, keyring, signer, classID, tokenID, quantity) {
  console.log("============== burn_nft ==============");

  await initApi(ws);
  await show_nft_by_account(ws, keyring, signer);
  const sk = keyring.addFromUri(signer);

  const call = Global_Api.tx.nftmart.burn(classID, tokenID, quantity);
  const feeInfo = await call.paymentInfo(sk);
  console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(sk, a);
  await b();

  await show_nft_by_account(ws, keyring, signer);
}

async function transfer_nfts(ws, keyring, tokens, from_raw, to) {
  console.log("============== transferring nft ==============");
  await initApi(ws);
  const from = keyring.addFromUri(from_raw);

  const call = Global_Api.tx.nftmart.transfer(ensureAddress(keyring, to), tokens);
  const feeInfo = await call.paymentInfo(from);
  console.log("The fee of the call: %s NMT.", feeInfo.partialFee / unit);

  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(from, a);
  await b();

  // console.log("from %s", from_raw);
  // await show_nft_by_account(ws, keyring, from_raw);
  // console.log("to %s", to);
  // await show_nft_by_account(ws, keyring, to);
}

async function show_class_by_account(ws, keyring, account) {
  await initApi(ws);
  const address = ensureAddress(keyring, account);
  const allClasses = await Global_Api.query.ormlNft.classes.entries();
  for (const c of allClasses) {
    let key = c[0];
    const len = key.length;
    key = key.buffer.slice(len - 4, len);
    const classID = new Uint32Array(key)[0];
    let clazz = c[1].toJSON();
    clazz.metadata = hexToUtf8(clazz.metadata.slice(2));
    try {
      clazz.metadata = JSON.parse(clazz.metadata);
    } catch (_e) {
    }
    clazz.classID = classID;
    clazz.adminList = await Global_Api.query.proxy.proxies(clazz.owner); // (Vec<ProxyDefinition>,BalanceOf)
    for (const a of clazz.adminList[0]) {
      if (a.delegate.toString() === address) {
        console.log("classInfo: %s", JSON.stringify(clazz));
      }
    }
  }
}

async function show_account_by_nft(ws, keyring, classId, tokenId) {
  await initApi(ws);
  const owners = await Global_Api.query.ormlNft.ownersByToken.entries([classId, tokenId]);
  for (let key of owners) {
    key = key[0];
    const len = key.length;
    key = key.buffer.slice(len - 32, len);
    const addr = keyring.encodeAddress(new Uint8Array(key));

    const accountInfo = await Global_Api.query.ormlNft.tokensByOwner(addr, [classId, tokenId]);
    console.log([classId, tokenId], addr.toString(), accountInfo.toString());
  }
}

async function show_nft_by_account(ws, keyring, account) {
  await initApi(ws);
  const nfts = await Global_Api.query.ormlNft.tokensByOwner.entries(ensureAddress(keyring, account));
  for (let clzToken of nfts) {
    const accountToken = clzToken[1];
    clzToken = clzToken[0];
    const len = clzToken.length;

    const classID = new Uint32Array(clzToken.slice(len - 4 - 8, len - 8))[0];
    const tokenID = Buffer.from(clzToken.slice(len - 8, len)).readBigUInt64LE();

    let nft = await Global_Api.query.ormlNft.tokens(classID, tokenID);
    print_nft(classID, tokenID, nft, accountToken);
  }
}

async function display_nft(classID) {
  let tokenCount = 0;
  let classInfo = await Global_Api.query.ormlNft.classes(classID);
  if (classInfo.isSome) {
    const nextTokenId = await Global_Api.query.ormlNft.nextTokenId(classID);
    console.log(`nextTokenId in classId ${classID} is ${nextTokenId}.`);
    classInfo = classInfo.unwrap();
    classInfo = classInfo.toJSON();
    try {
      classInfo.metadata = hexToUtf8(classInfo.metadata.slice(2));
      classInfo.metadata = JSON.parse(classInfo.metadata);
    } catch (e) {
    }

    try {
      classInfo.data.name = hexToUtf8(classInfo.data.name.slice(2));
      classInfo.data.name = JSON.parse(classInfo.data.name);
    } catch (e) {
    }
    try {
      classInfo.data.description = hexToUtf8(classInfo.data.description.slice(2));
      classInfo.data.description = JSON.parse(classInfo.data.description);
    } catch (e) {
    }

    const accountInfo = await Global_Api.query.system.account(classInfo.owner);
    console.log("classInfo: %s", JSON.stringify(classInfo));
    console.log("classOwner: %s", accountInfo.toString());
    for (let i = 0; i < nextTokenId; i++) {
      let nft = await Global_Api.query.ormlNft.tokens(classID, i);
      if (nft.isSome) {
        print_nft(classID, i, nft);
        tokenCount++;
      }
    }
  }
  console.log(`The token count of class ${classID} is ${tokenCount}.`);
}

async function show_nft_by_class(ws, classID) {
  await initApi(ws);
  if (classID === undefined) { // find all nfts
    const allClasses = await Global_Api.query.ormlNft.classes.entries();
    for (const c of allClasses) {
      let key = c[0];
      const len = key.length;
      key = key.buffer.slice(len - 4, len);
      const classID = new Uint32Array(key)[0];
      await display_nft(classID);
    }
  } else {
    await display_nft(classID);
  }
}

async function mint_nft_by_proxy(ws, keyring, admin, classID, quantity, royalty_rate) {
  console.log("============== mint_nft_by_proxy ==============");

  await initApi(ws);
  admin = keyring.addFromUri(admin);
  const nftMetadata = 'demo nft metadata';

  const call = Global_Api.tx.nftmart.proxyMint(admin.address, classID,
    nftMetadata, quantity, royalty_rate);

  const feeInfo = await call.paymentInfo(admin);
  console.log("The fee of the call: %s NMT.", feeInfo.partialFee / unit);

  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(admin, a);
  await b();
}

async function mint_nft(ws, keyring, admin, classID, quantity, royalty_rate) {
  console.log("============== minting nft ==============");
  await initApi(ws);
  admin = keyring.addFromUri(admin);
  const classInfo = await Global_Api.query.ormlNft.classes(classID);
  if (classInfo.isSome) {
    const ownerOfClass = classInfo.unwrap().owner.toString();
    const nftMetadata = 'demo nft metadata';
    const balancesNeeded = await nftDeposit(nftMetadata);
    if (balancesNeeded === null) {
      return;
    }
    // royalty_rate = null; // follow the config of royalty_rate in class.
    // royalty_rate = float;
    console.log("royalty_rate: %s", royalty_rate);
    const txs = [
      // make sure `ownerOfClass` has sufficient balances to mint nft.
      Global_Api.tx.balances.transfer(ownerOfClass, balancesNeeded),
      // mint some nfts and transfer to admin.address.
      Global_Api.tx.proxy.proxy(ownerOfClass, null,
        Global_Api.tx.nftmart.mint(admin.address, classID, nftMetadata, quantity, royalty_rate)),
    ];
    const batchExtrinsic = Global_Api.tx.utility.batchAll(txs);
    const feeInfo = await batchExtrinsic.paymentInfo(admin);
    console.log("fee of batchExtrinsic: %s NMT", feeInfo.partialFee / unit);

    let [a, b] = waitTx(Global_ModuleMetadata);
    await batchExtrinsic.signAndSend(admin, a);
    await b();
  } else {
    console.log("class %s not found.", classID);
  }
}

async function add_class_admin(ws, keyring, admin, classId, newAdmin) {
  console.log("============== adding class admin ==============");
  await initApi(ws);
  admin = keyring.addFromUri(admin);
  newAdmin = ensureAddress(keyring, newAdmin);
  let classInfo = await Global_Api.query.ormlNft.classes(classId);
  if (classInfo.isSome) {
    classInfo = classInfo.unwrap();
    const ownerOfClass = classInfo.owner;
    console.log(ownerOfClass.toString());
    const balancesNeeded = await proxyDeposit(1);
    if (balancesNeeded === null) {
      return;
    }
    console.log("adding a class admin needs to reserve %s NMT", balancesNeeded / unit);
    const txs = [
      // make sure `ownerOfClass` has sufficient balances.
      Global_Api.tx.balances.transfer(ownerOfClass, balancesNeeded),
      // Add `newAdmin` as a new admin.
      Global_Api.tx.proxy.proxy(ownerOfClass, null, Global_Api.tx.proxy.addProxy(newAdmin, 'Any', 0)),
      // Global_Api.tx.proxy.proxy(ownerOfClass, null, Global_Api.tx.proxy.removeProxy(newAdmin, 'Any', 0)), to remove an admin
    ];
    const batchExtrinsic = Global_Api.tx.utility.batchAll(txs);
    const feeInfo = await batchExtrinsic.paymentInfo(admin);
    console.log("fee of batchExtrinsic: %s NMT", feeInfo.partialFee / unit);

    let [a, b] = waitTx(Global_ModuleMetadata);
    await batchExtrinsic.signAndSend(admin, a);
    await b();
  }
}

async function show_class(ws) {
  await initApi(ws);
  let classCount = 0;
  const allClasses = await Global_Api.query.ormlNft.classes.entries();
  let all = [];
  for (const c of allClasses) {
    let key = c[0];
    const len = key.length;
    key = key.buffer.slice(len - 4, len);
    const classID = new Uint32Array(key)[0];
    let clazz = c[1].toHuman();
    let clazzJson = c[1].toJSON();
    clazz.metadata = hexToUtf8(clazzJson.metadata.slice(2));
    try {
      clazz.metadata = JSON.parse(clazzJson.metadata);
    } catch (_e) {
    }
    clazz.data.royalty_rate = perU16ToFloat(clazzJson.data.royalty_rate);
    clazz.classID = classID;
    clazz.adminList = await Global_Api.query.proxy.proxies(clazz.owner);
    all.push(JSON.stringify(clazz));
    classCount++;
  }
  console.log("%s", all);
  console.log("class count: %s", classCount);
  console.log("nextClassId: %s", await Global_Api.query.ormlNft.nextClassId());
}

async function add_whitelist(ws, keyring, sudo, account) {
  console.log("============== add_whitelist ==============");
  // usage: node nft-apis.mjs add-whitelist //Alice 63dHdZZMdgFeHs544yboqnVvrnAaTRdPWPC1u2aZjpC5HTqx
  await initApi(ws);
  sudo = keyring.addFromUri(sudo);
  account = ensureAddress(keyring, account);
  // const call = Global_Api.tx.sudo.sudo(Global_Api.tx.config.removeWhitelist(account.address)); to remove account from whitelist
  const call = Global_Api.tx.sudo.sudo(Global_Api.tx.nftmartConf.addWhitelist(account));
  const feeInfo = await call.paymentInfo(sudo.address);
  console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
  let [a, b] = waitTx(Global_ModuleMetadata);
  await call.signAndSend(sudo, a);
  await b();
}

async function show_whitelist(ws, keyring) {
  await initApi(ws);
  const all = await Global_Api.query.nftmartConf.accountWhitelist.entries();
  for (const account of all) {
    let key = account[0];
    const len = key.length;
    key = key.buffer.slice(len - 32, len);
    const addr = keyring.encodeAddress(new Uint8Array(key));
    console.log("%s", addr);
  }
}

async function create_class(ws, keyring, signer) {
  console.log("============== creating class ==============");
  await initApi(ws);
  signer = keyring.addFromUri(signer);

  const name = 'demo class name';
  const description = 'demo class description';
  const metadata = 'demo class metadata';
  const royalty_rate = float2PerU16(0.20);

  const deposit = await classDeposit(metadata, name, description);
  console.log("create class deposit %s", deposit);

  // 	Transferable = 0b00000001,
  // 	Burnable = 0b00000010,
  let [a, b] = waitTx(Global_ModuleMetadata);
  await Global_Api.tx.nftmart.createClass(metadata, name, description, royalty_rate, 1 | 2).signAndSend(signer, a);
  await b();
}

main().then(r => {
  process.exit();
}).catch(err => {
  console.log(err);
  process.exit();
});
