import { PubkyAppPost, PubkyAppPostKind, PubkySpecsBuilder, PubkyAppPostEmbed, PubkyAppWatchlist, PubkyAppMarketplaceOrderReceipt, PubkyAppMarketplaceDrop, postUriBuilder, bookmarkUriBuilder, followUriBuilder, userUriBuilder, watchlistUriBuilder, orderReceiptUriBuilder, dropUriBuilder, parseOrderReceiptAttestation, verifyOrderReceiptAttestation, parseDropEditionAttestation, verifyDropEditionAttestation, getValidMimeTypes } from "./index.js";
import { createRequire } from "node:module";
import { generateKeyPairSync, sign as cryptoSign } from "node:crypto";
import assert from "assert";

const require = createRequire(import.meta.url);
const { validationLimits, getValidationLimits } = require("./validationLimits.cjs");
const validationLimitsJson = require("./validationLimits.json");

const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

describe("PubkySpecs Example Objects Tests", () => {
  let specsBuilder;

  beforeEach(() => {
    specsBuilder = new PubkySpecsBuilder(OTTO);
  });

  describe("User Pubky-app-specs", () => {
    it("should create user with correct properties", () => {
      const { user, meta: userMeta } = specsBuilder.createUser(
        "Alice Smith",
        "Software Developer", 
        null, 
        null, 
        "active"
      );

      // Test meta properties
      assert.ok(userMeta.url, "User should have a URL");
      assert.ok(userMeta.url.includes(OTTO), "URL should contain user ID");
      assert.ok(userMeta.url.includes("profile.json"), "URL should point to profile.json");

      // Test user object content
      const userJson = user.toJson();
      assert.strictEqual(userJson.name, "Alice Smith", "User name should match");
      assert.strictEqual(userJson.bio, "Software Developer", "User bio should match");
      assert.strictEqual(userJson.status, "active", "User status should match");
    });

    it("cannot create user with name too short", () => {
      assert.throws(
        () => {
          specsBuilder.createUser("AB", null, null, null, null); // 2 chars, min is 3
        },
        (err) => {
          const msg = err instanceof Error ? err.message : String(err);
          assert.ok(
            msg.includes("Invalid name length"),
            `Expected 'Invalid name length' error, got: "${msg}"`
          );
          return true;
        },
        "Expected validation error for name too short"
      );
    });

    it("should accept emoji name at max length (50 chars)", () => {
      const emojiName = "🔥".repeat(50); // 50 emoji = 50 Unicode chars (but many more bytes)
      assert.strictEqual([...emojiName].length, 50, "Should be 50 Unicode characters");

      const { user } = specsBuilder.createUser(emojiName, null, null, null, null);
      const userJson = user.toJson();
      assert.strictEqual(userJson.name, emojiName, "Emoji name should be preserved");
    });
  });

  describe("Post Pubky-app-specs", () => {
    it("should create basic post with correct properties", () => {
      const postContent = "Hello, Pubky world! This is my first post."
      const { post, meta } = specsBuilder.createPost(postContent, PubkyAppPostKind.Short);

      // Test meta properties
      assert.ok(meta.id, "Post should have an ID");
      assert.ok(meta.url, "Post should have a URL");
      const postChunks = meta.url.split("/")
      assert.strictEqual(postChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(postChunks[5], "posts", "URL should contain posts path");
      assert.strictEqual(postChunks[6], meta.id, "URL should contain post ID");

      // Test post content
      const postJson = post.toJson();
      assert.strictEqual(postJson.content, postContent, "Post content should match");
      assert.strictEqual(postJson.kind, "short", "Post kind should match");
    });

    it("should create reply post with parent reference", () => {
      const parentPostUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SSE3B1FQ0`
      const parentPostUri = postUriBuilder(RIO, "0033SSE3B1FQ0")
      assert.strictEqual(parentPostUri, parentPostUriRaw, "Parent post URI should match");

      const { post: replyPost } = specsBuilder.createPost(
        "This is a reply to the first post!",
        PubkyAppPostKind.Short,
        parentPostUriRaw
      );

      // Test reply content
      const replyJson = replyPost.toJson();
      assert.strictEqual(replyJson.parent, parentPostUriRaw, "Reply should reference parent URL");
    });

    it("should create repost with embed", () => {
      const embedUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`
      const embedUriFromBuilder = postUriBuilder(RIO, "0033SREKPC4N0")
      assert.strictEqual(embedUriFromBuilder, embedUriRaw, "Embed URI should match");

      const embed = new PubkyAppPostEmbed(embedUriRaw, PubkyAppPostKind.Video);
      const { post: repost } = specsBuilder.createPost(
        "This is a repost to random post!",
        PubkyAppPostKind.Short,
        null,
        embed
      );

      // Test repost content
      const repostJson = repost.toJson();
      assert.ok(repostJson.embed, "Repost should have embed");
      assert.strictEqual(repostJson.embed.uri, embedUriRaw, "Embed URI should match");
      assert.strictEqual(repostJson.embed.kind, "video", "Embed kind should match");
    });

    it("cannot create post with too many attachments", () => {
      const attachments = [
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ52G`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ53H`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ54I`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55J`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55K`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55L`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55M`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55N`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55O`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55P`,
        `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ55A`, // 11th attachment exceeds limit
      ];

      assert.throws(
        () => {
          specsBuilder.createPost(
            "Post with too many attachments",
            PubkyAppPostKind.Image,
            null,
            null,
            attachments
          );
        },
        (err) => {
          const msg = err instanceof Error ? err.message : String(err);
          assert.ok(
            msg.includes("Too many attachments"),
            `Expected 'Too many attachments' error, got: "${msg}"`
          );
          return true;
        },
        "Expected validation error for too many attachments"
      );
    });

    describe("Post lock", () => {
      const validLockUrl = `pubky://${RIO}/pub/locks/0034A0X7NJ52G`;

      it("should create locked post with valid pubky lock URL", () => {
        const postContent = "Visible preview for locked content";
        const { post } = specsBuilder.createPost(
          postContent,
          PubkyAppPostKind.Long,
          null,
          null,
          null,
          validLockUrl
        );

        assert.strictEqual(post.lock, validLockUrl, "lock getter should return lock URL");
        const postJson = post.toJson();
        assert.strictEqual(postJson.content, postContent, "Post content should match");
        assert.strictEqual(postJson.lock, validLockUrl, "toJson should include lock URL");
      });

      it("should create unlocked post when lock is omitted", () => {
        const { post } = specsBuilder.createPost("Hello", PubkyAppPostKind.Short);
        const lock = post.lock;
        assert.ok(
          lock === null || lock === undefined,
          "createPost should produce unlocked post"
        );
        const postJson = post.toJson();
        assert.ok(
          postJson.lock === null || postJson.lock === undefined,
          "toJson should not include lock for unlocked post"
        );
      });

      it("should deserialize post without lock field as unlocked", () => {
        const post = PubkyAppPost.fromJson({
          content: "Hello World!",
          kind: "short",
          parent: null,
          embed: null,
          attachments: null,
        });
        const lock = post.lock;
        assert.ok(
          lock === null || lock === undefined,
          "fromJson without lock should deserialize as unlocked"
        );
      });

      it("cannot create post with non-pubky lock URL", () => {
        assert.throws(
          () => {
            specsBuilder.createPost(
              "Preview",
              PubkyAppPostKind.Short,
              null,
              null,
              null,
              "https://locks.example.com/session/0034A0X7NJ52G"
            );
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(
              msg.includes("pubky://"),
              `Expected pubky:// scheme error, got: "${msg}"`
            );
            return true;
          },
          "Expected validation error for non-pubky lock URL"
        );
      });

      it("cannot create post with hostless lock URL", () => {
        assert.throws(
          () => {
            specsBuilder.createPost(
              "Preview",
              PubkyAppPostKind.Short,
              null,
              null,
              null,
              "pubky:lock-id"
            );
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(msg.includes("host"), `Expected host error, got: "${msg}"`);
            return true;
          },
          "Expected validation error for hostless lock URL"
        );
      });
    });

    describe("Collection posts", () => {
      const collectionItemUri = `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`;
      const coverImageUrl = `pubky://${RIO}/pub/pubky.app/files/0034A0X7NJ52G`;

      it("should create collection post with JSON envelope content", () => {
        assert.strictEqual(
          typeof specsBuilder.createCollectionPost,
          "function",
          "PubkySpecsBuilder should expose createCollectionPost"
        );

        const { post, meta } = specsBuilder.createCollectionPost(
          "Favorite posts",
          "Posts worth revisiting",
          [collectionItemUri],
          coverImageUrl
        );

        assert.ok(meta.id, "Collection post should have an ID");
        assert.ok(meta.url, "Collection post should have a URL");
        const postChunks = meta.url.split("/");
        assert.strictEqual(postChunks[2], OTTO, "URL should contain user ID");
        assert.strictEqual(postChunks[5], "posts", "URL should contain posts path");
        assert.strictEqual(postChunks[6], meta.id, "URL should contain post ID");

        const postJson = post.toJson();
        assert.strictEqual(postJson.kind, "collection", "Post kind should be collection");
        assert.ok(
          postJson.attachments === null || postJson.attachments === undefined,
          "Collection items should not be stored in post.attachments"
        );

        const envelope = JSON.parse(postJson.content);
        assert.strictEqual(envelope.name, "Favorite posts", "Collection name should match");
        assert.strictEqual(
          envelope.description,
          "Posts worth revisiting",
          "Collection description should match"
        );
        assert.deepStrictEqual(envelope.items, [collectionItemUri], "Collection items should match");
        assert.strictEqual(envelope.cover_image, coverImageUrl, "Collection cover image should match");
      });

      it("cannot create collection post with too many items", () => {
        assert.strictEqual(
          typeof specsBuilder.createCollectionPost,
          "function",
          "PubkySpecsBuilder should expose createCollectionPost"
        );

        const tooManyItems = Array.from(
          { length: validationLimits.collectionItemsMaxCount + 1 },
          (_, index) => `pubky://${RIO}/pub/pubky.app/posts/${String(index).padStart(13, "0")}`
        );

        assert.throws(
          () => {
            specsBuilder.createCollectionPost("Too many", null, tooManyItems, null);
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(
              msg.includes(`${validationLimits.collectionItemsMaxCount} items`),
              `Expected collection item limit error, got: "${msg}"`
            );
            return true;
          },
          "Expected validation error for too many collection items"
        );
      });
    });
  });

  describe("Bookmark Pubky-app-specs", () => {
    it("should create bookmark with correct properties", () => {
      const postUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`

      const { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(postUriRaw);
      const bookmarkUriFromBuilder = bookmarkUriBuilder(OTTO, bookmarkMeta.id)
      assert.strictEqual(bookmarkUriFromBuilder, bookmarkMeta.url, "Bookmark URI should match");

      // Test meta properties
      assert.ok(bookmarkMeta.id, "Bookmark should have an ID");
      assert.ok(bookmarkMeta.url, "Bookmark should have a URL");
      const bookmarkChunks = bookmarkMeta.url.split("/")
      assert.strictEqual(bookmarkChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(bookmarkChunks[5], "bookmarks", "URL should contain bookmarks path");
      assert.strictEqual(bookmarkChunks[6], bookmarkMeta.id, "URL should contain bookmark ID");

      // Test bookmark content
      const bookmarkJson = bookmark.toJson();
      assert.strictEqual(bookmarkJson.uri, postUriRaw, "Bookmark URI should match");
      assert.ok(bookmarkJson.created_at, "Bookmark should have created_at timestamp");
      assert.ok(typeof bookmarkJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Follow Pubky-app-specs", () => {
    it("should create follow with correct properties", () => {
      const { follow, meta: followMeta } = specsBuilder.createFollow(RIO);
      const followUriFromBuilder = followUriBuilder(OTTO, RIO)
      assert.strictEqual(followUriFromBuilder, followMeta.url, "Follow URI should match");

      // Test meta properties
      assert.strictEqual(followMeta.id, RIO, "Follow ID should be the user being followed");
      assert.ok(followMeta.url, "Follow should have a URL");
      const followChunks = followMeta.url.split("/")
      assert.strictEqual(followChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(followChunks[5], "follows", "URL should contain follows path");
      assert.strictEqual(followChunks[6], RIO, "URL should contain follow ID");

      // Test follow content
      const followJson = follow.toJson();
      assert.ok(followJson.created_at, "Follow should have created_at timestamp");
      assert.ok(typeof followJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Tag Pubky-app-specs", () => {
    it("should create tag with correct properties", () => {
      const userUriRaw = `pubky://${OTTO}/pub/pubky.app/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO)
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const { tag, meta: tagMeta } = specsBuilder.createTag(userUriRaw, "otto");

      // Test meta properties
      assert.ok(tagMeta.id, "Tag should have an ID");
      assert.ok(tagMeta.url, "Tag should have a URL");
      const tagChunks = tagMeta.url.split("/")
      assert.strictEqual(tagChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(tagChunks[5], "tags", "URL should contain tags path");
      assert.strictEqual(tagChunks[6], tagMeta.id, "URL should contain tag ID");

      // Test tag content
      const tagJson = tag.toJson();
      assert.strictEqual(tagJson.uri, userUriRaw, "Tag URI should match");
      assert.strictEqual(tagJson.label, "otto", "Tag label should match");
      assert.ok(tagJson.created_at, "Tag should have created_at timestamp");
      assert.ok(typeof tagJson.created_at === "number", "created_at should be a number");
    });
    it("cannot create a tag with invalid characters (comma, colon, space)", () => {
      const userUriRaw = `pubky://${OTTO}/pub/pubky.app/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO);
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const invalidCases = [
        { label: "otto,rio", invalidChar: ",", isWhitespace: false },
        { label: "otto:rio", invalidChar: ":", isWhitespace: false },
        { label: "otto rio", invalidChar: " ", isWhitespace: true },
      ];

      invalidCases.forEach(({ label, invalidChar, isWhitespace }) => {
        assert.throws(
          () => {
            specsBuilder.createTag(userUriRaw, label);
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);

            if (isWhitespace) {
              // Whitespace has a different error message format
              assert.strictEqual(
                msg,
                `Validation Error: Tag '${label}' contains whitespace characters`,
                `Unexpected error message for whitespace: "${msg}"`
              );
            } else {
              assert.strictEqual(
                msg,
                `Validation Error: Tag '${label}' contains invalid character: ${invalidChar}`,
                `Unexpected error message for invalid char '${invalidChar}': "${msg}"`
              );
            }

            return true;
          },
          `Expected validation error when creating tag with invalid char '${invalidChar}' in label`
        );
      });
    });
  });

  describe("Mute Pubky-app-specs", () => {
    it("should create mute with correct properties", () => {
      const { mute, meta: muteMeta } = specsBuilder.createMute(RIO);

      // Test meta properties
      assert.ok(muteMeta.id, "Mute should have an ID");
      assert.ok(muteMeta.url, "Mute should have a URL");
      const muteChunks = muteMeta.url.split("/")
      assert.strictEqual(muteChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(muteChunks[5], "mutes", "URL should contain mutes path");
      assert.strictEqual(muteChunks[6], muteMeta.id, "URL should contain mute ID");

      // Test mute content
      const muteJson = mute.toJson();
      assert.ok(muteJson.created_at, "Mute should have created_at timestamp");
      assert.ok(typeof muteJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("LastRead Pubky-app-specs", () => {
    it("should create last_read with correct properties", () => {
      const { last_read, meta: lastReadMeta } = specsBuilder.createLastRead(RIO);

      // Test meta properties
      assert.ok(lastReadMeta.url, "LastRead should have a URL");
      const lastReadChunks = lastReadMeta.url.split("/")
      assert.strictEqual(lastReadChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(lastReadChunks[5], "last_read", "URL should contain last_read path");
      assert.strictEqual(lastReadChunks.length, 6, "URL should have 6 segments");

      // Test last_read content
      const lastReadJson = last_read.toJson();
      assert.ok(lastReadJson.timestamp, "LastRead should have timestamp");
      assert.ok(typeof lastReadJson.timestamp === "number", "timestamp should be a number");
    });
  });

  describe("Watchlist Pubky-app-specs (private)", () => {
    const watchlistBody = () => ({
      schemaVersion: 1,
      recordType: "watchlist",
      ownerPubky: OTTO,
      revision: 1,
      createdAt: "2025-01-01T00:00:00Z",
      updatedAt: "2025-01-02T00:00:00Z",
      items: [
        { listingOwnerPubky: RIO, listingId: "0032SSN7Q4EVG", watchedAtMs: 1735689600000 },
      ],
      tombstones: [
        { listingOwnerPubky: RIO, listingId: "0032SSN7Q4EVH", removedAtMs: 1735776000000 },
      ],
    });

    it("should create a private watchlist under /priv", () => {
      const { watchlist, meta } = specsBuilder.createWatchlist(watchlistBody());

      assert.strictEqual(
        meta.path,
        "/priv/pubky.app/marketplace/v1/watchlist.json",
        "Watchlist path must live under /priv"
      );
      assert.strictEqual(
        meta.url,
        `pubky://${OTTO}/priv/pubky.app/marketplace/v1/watchlist.json`,
        "Watchlist URL must be the owner's private URI"
      );
      assert.strictEqual(meta.url, watchlistUriBuilder(OTTO), "URI builder must agree with meta");

      const json = watchlist.toJson();
      assert.strictEqual(json.recordType, "watchlist");
      assert.strictEqual(json.items[0].watchedAtMs, 1735689600000);
      assert.strictEqual(json.tombstones[0].removedAtMs, 1735776000000);

      const roundtrip = PubkyAppWatchlist.fromJson(json);
      assert.strictEqual(roundtrip.toJson().items[0].listingId, "0032SSN7Q4EVG");
    });

    it("should reject a listing key present in both items and tombstones", () => {
      const body = watchlistBody();
      body.tombstones.push({
        listingOwnerPubky: RIO,
        listingId: "0032SSN7Q4EVG",
        removedAtMs: 1,
      });
      assert.throws(() => specsBuilder.createWatchlist(body));
    });
  });

  describe("Marketplace order receipt Pubky-app-specs (private)", () => {
    const RECEIPT_ID = "a7fc7d5d-0b2a-4083-b278-47193f8fe536";
    const ORDER_ID = "0e9c2c4a-91d6-4a4e-8db3-2f14c1e8b7aa";
    const PAID_AT = "2026-01-02T03:04:05Z";

    const receiptBody = () => ({
      schemaVersion: 1,
      recordType: "order_receipt",
      ownerPubky: OTTO,
      revision: 1,
      createdAt: PAID_AT,
      updatedAt: PAID_AT,
      role: "buyer",
      receiptId: RECEIPT_ID,
      orderId: ORDER_ID,
      buyerPubky: OTTO,
      sellerPubky: RIO,
      total: { amountMinor: 12000, currency: "USD", exponent: 2 },
      paidAt: PAID_AT,
      receiptAttestation: "a".repeat(64),
    });

    // z-base-32 encoding of raw bytes (the pubky encoding of Ed25519 keys).
    const zbase32 = (bytes) => {
      const alphabet = "ybndrfg8ejkmcpqxot1uwisza345h769";
      let bits = 0, accumulator = 0, out = "";
      for (const byte of bytes) {
        accumulator = (accumulator << 8) | byte;
        bits += 8;
        while (bits >= 5) {
          bits -= 5;
          out += alphabet[(accumulator >> bits) & 31];
        }
      }
      if (bits > 0) out += alphabet[(accumulator << (5 - bits)) & 31];
      return out;
    };

    const b64u = (data) => Buffer.from(data).toString("base64url");

    it("should create a private order receipt under /priv with id = receiptId", () => {
      const { order_receipt, meta } = specsBuilder.createMarketplaceOrderReceipt(receiptBody());

      assert.strictEqual(meta.id, RECEIPT_ID, "Meta id must be the receipt UUID");
      assert.strictEqual(
        meta.path,
        `/priv/pubky.app/marketplace/v1/receipts/${RECEIPT_ID}`,
        "Receipt path must live under /priv"
      );
      assert.strictEqual(
        meta.url,
        `pubky://${OTTO}/priv/pubky.app/marketplace/v1/receipts/${RECEIPT_ID}`,
        "Receipt URL must be the owner's private URI"
      );
      assert.strictEqual(
        meta.url,
        orderReceiptUriBuilder(OTTO, RECEIPT_ID),
        "URI builder must agree with meta"
      );

      const json = order_receipt.toJson();
      assert.strictEqual(json.recordType, "order_receipt");
      assert.strictEqual(json.role, "buyer");
      assert.strictEqual(json.total.amountMinor, 12000);

      const roundtrip = PubkyAppMarketplaceOrderReceipt.fromJson(json);
      assert.strictEqual(roundtrip.toJson().orderId, ORDER_ID);
    });

    it("should reject an owner/role mismatch", () => {
      const body = receiptBody();
      body.role = "seller"; // owner OTTO is the buyer, not the seller
      assert.throws(() => specsBuilder.createMarketplaceOrderReceipt(body));
    });

    it("should verify a real Ed25519 receipt attestation end to end", () => {
      const { publicKey, privateKey } = generateKeyPairSync("ed25519");
      const rawPublicKey = Buffer.from(publicKey.export({ format: "jwk" }).x, "base64url");
      const iss = zbase32(rawPublicKey);
      assert.strictEqual(iss.length, 52, "iss must be a 52-char pubky");

      const claims = {
        v: 1,
        iss,
        buyer: OTTO,
        seller: RIO,
        order: ORDER_ID,
        receipt: RECEIPT_ID,
        total_minor: 12000,
        currency: "USD",
        exponent: 2,
        paid_at: PAID_AT,
        iat: Date.parse(PAID_AT) / 1000,
      };
      const header = b64u(JSON.stringify({ alg: "EdDSA", typ: "pubky-order-receipt+v1" }));
      const payload = b64u(JSON.stringify(claims));
      const signature = cryptoSign(null, Buffer.from(`${header}.${payload}`), privateKey);
      const jws = `${header}.${payload}.${b64u(signature)}`;

      const parsed = parseOrderReceiptAttestation(jws);
      assert.strictEqual(parsed.iss, iss, "Parsed claims must carry the attestor pubky");

      const body = receiptBody();
      body.receiptAttestation = jws;
      const verified = verifyOrderReceiptAttestation(body);
      assert.strictEqual(verified.receipt, RECEIPT_ID, "Verified claims must bind the receipt");
      assert.strictEqual(verified.total_minor, 12000, "Verified claims must bind the total");
    });

    it("should reject an attestation that does not verify", () => {
      // Structurally charset-valid but not a real JWS.
      assert.throws(() => verifyOrderReceiptAttestation(receiptBody()));
      // Structurally invalid compact form.
      assert.throws(() => parseOrderReceiptAttestation("not.a.jws"));
    });

    it("should keep parsing a .7-shaped receipt without the drop-edition fields", () => {
      const { order_receipt } = specsBuilder.createMarketplaceOrderReceipt(receiptBody());
      const json = order_receipt.toJson();
      assert.ok(!("editionAttestation" in json) || json.editionAttestation == null,
        "Absent editionAttestation must stay absent");
      assert.ok(!("drop" in json) || json.drop == null, "Absent drop must stay absent");

      const roundtrip = PubkyAppMarketplaceOrderReceipt.fromJson(json).toJson();
      assert.ok(!("editionAttestation" in roundtrip) || roundtrip.editionAttestation == null,
        "Roundtrip must not invent editionAttestation");
      assert.ok(!("drop" in roundtrip) || roundtrip.drop == null, "Roundtrip must not invent drop");
    });

    it("should reject editionAttestation without the drop object (and vice versa)", () => {
      const lonelyAttestation = receiptBody();
      lonelyAttestation.editionAttestation = "b".repeat(64);
      assert.throws(() => specsBuilder.createMarketplaceOrderReceipt(lonelyAttestation));

      const lonelyDrop = receiptBody();
      lonelyDrop.drop = { dropId: "spring-drop-01", edition: 7, of: 500 };
      assert.throws(() => specsBuilder.createMarketplaceOrderReceipt(lonelyDrop));
    });

    it("should accept a receipt carrying both drop-edition fields", () => {
      const body = receiptBody();
      body.editionAttestation = "b".repeat(64);
      body.drop = { dropId: "spring-drop-01", edition: 7, of: 500 };
      const { order_receipt } = specsBuilder.createMarketplaceOrderReceipt(body);
      const json = order_receipt.toJson();
      assert.strictEqual(json.drop.dropId, "spring-drop-01");
      assert.strictEqual(json.drop.edition, 7);
      assert.strictEqual(json.drop.of, 500);
      assert.strictEqual(json.editionAttestation, "b".repeat(64));
    });
  });

  describe("Marketplace drop Pubky-app-specs", () => {
    const DROP_ID = "spring-drop-01";

    const dropBody = () => ({
      schemaVersion: 1,
      recordType: "drop",
      ownerPubky: OTTO,
      revision: 1,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-02T00:00:00Z",
      dropId: DROP_ID,
      title: "Spring boot drop",
      description: "Limited spring release.",
      media: [`pubky://${OTTO}/pub/pubky.app/marketplace/v1/media/drop_banner`],
      format: "fcfs",
      startsAt: "2026-02-01T00:00:00Z",
      endsAt: "2026-02-02T00:00:00Z",
      listingIds: ["listing_01", "listing_02"],
      totalQuantity: 500,
      perBuyerLimit: 2,
      stockDisplay: "bands",
    });

    it("should create a public drop with id = dropId", () => {
      const { marketplace_drop, meta } = specsBuilder.createMarketplaceDrop(dropBody());

      assert.strictEqual(meta.id, DROP_ID, "Meta id must be the dropId");
      assert.strictEqual(
        meta.path,
        `/pub/pubky.app/marketplace/v1/drops/${DROP_ID}`,
        "Drop path must live under /pub so it can be indexed"
      );
      assert.strictEqual(
        meta.url,
        `pubky://${OTTO}/pub/pubky.app/marketplace/v1/drops/${DROP_ID}`,
        "Drop URL must be the owner's public URI"
      );
      assert.strictEqual(meta.url, dropUriBuilder(OTTO, DROP_ID), "URI builder must agree with meta");

      const json = marketplace_drop.toJson();
      assert.strictEqual(json.recordType, "drop");
      assert.strictEqual(json.format, "fcfs");
      assert.strictEqual(json.stockDisplay, "bands");
      assert.strictEqual(json.totalQuantity, 500);
      assert.strictEqual(json.perBuyerLimit, 2);

      const roundtrip = PubkyAppMarketplaceDrop.fromJson(json);
      assert.strictEqual(roundtrip.toJson().dropId, DROP_ID);
    });

    it("should accept an open-ended drop without endsAt", () => {
      const body = dropBody();
      delete body.endsAt;
      const { marketplace_drop } = specsBuilder.createMarketplaceDrop(body);
      const json = marketplace_drop.toJson();
      assert.ok(!("endsAt" in json) || json.endsAt == null, "Absent endsAt must stay absent");
    });

    it("should reject endsAt at or before startsAt", () => {
      const body = dropBody();
      body.endsAt = body.startsAt;
      assert.throws(() => specsBuilder.createMarketplaceDrop(body));
    });

    it("should reject perBuyerLimit above totalQuantity", () => {
      const body = dropBody();
      body.totalQuantity = 3;
      body.perBuyerLimit = 4;
      assert.throws(() => specsBuilder.createMarketplaceDrop(body));
    });

    it("should reject unknown format values (closed-world enum)", () => {
      const body = dropBody();
      body.format = "auction";
      assert.throws(() => specsBuilder.createMarketplaceDrop(body));
    });

    it("should reject duplicate listingIds", () => {
      const body = dropBody();
      body.listingIds = ["listing_01", "listing_01"];
      assert.throws(() => specsBuilder.createMarketplaceDrop(body));
    });
  });

  describe("Drop edition attestation Pubky-app-specs", () => {
    const RECEIPT_ID = "a7fc7d5d-0b2a-4083-b278-47193f8fe536";
    const ORDER_ID = "0e9c2c4a-91d6-4a4e-8db3-2f14c1e8b7aa";
    const DROP_ID = "spring-drop-01";
    const PAID_AT = "2026-01-02T03:04:05Z";

    // z-base-32 encoding of raw bytes (the pubky encoding of Ed25519 keys).
    const zbase32 = (bytes) => {
      const alphabet = "ybndrfg8ejkmcpqxot1uwisza345h769";
      let bits = 0, accumulator = 0, out = "";
      for (const byte of bytes) {
        accumulator = (accumulator << 8) | byte;
        bits += 8;
        while (bits >= 5) {
          bits -= 5;
          out += alphabet[(accumulator >> bits) & 31];
        }
      }
      if (bits > 0) out += alphabet[(accumulator << (5 - bits)) & 31];
      return out;
    };

    const b64u = (data) => Buffer.from(data).toString("base64url");

    const receiptWithDrop = (editionAttestation) => ({
      schemaVersion: 1,
      recordType: "order_receipt",
      ownerPubky: OTTO,
      revision: 1,
      createdAt: PAID_AT,
      updatedAt: PAID_AT,
      role: "buyer",
      receiptId: RECEIPT_ID,
      orderId: ORDER_ID,
      buyerPubky: OTTO,
      sellerPubky: RIO,
      total: { amountMinor: 12000, currency: "USD", exponent: 2 },
      paidAt: PAID_AT,
      receiptAttestation: "a".repeat(64),
      editionAttestation,
      drop: { dropId: DROP_ID, edition: 7, of: 500 },
    });

    it("should verify a real Ed25519 drop edition attestation end to end", () => {
      const { publicKey, privateKey } = generateKeyPairSync("ed25519");
      const rawPublicKey = Buffer.from(publicKey.export({ format: "jwk" }).x, "base64url");
      const iss = zbase32(rawPublicKey);
      assert.strictEqual(iss.length, 52, "iss must be a 52-char pubky");

      const claims = {
        v: 1,
        iss,
        buyer: OTTO,
        seller: RIO,
        drop: DROP_ID,
        edition: 7,
        of: 500,
        receipt: RECEIPT_ID,
        iat: Date.parse(PAID_AT) / 1000,
      };
      const header = b64u(JSON.stringify({ alg: "EdDSA", typ: "pubky-drop-edition+v1" }));
      const payload = b64u(JSON.stringify(claims));
      const signature = cryptoSign(null, Buffer.from(`${header}.${payload}`), privateKey);
      const jws = `${header}.${payload}.${b64u(signature)}`;

      const parsed = parseDropEditionAttestation(jws);
      assert.strictEqual(parsed.iss, iss, "Parsed claims must carry the attestor pubky");
      assert.strictEqual(parsed.edition, 7, "Parsed claims must carry the edition");

      const verified = verifyDropEditionAttestation(receiptWithDrop(jws));
      assert.strictEqual(verified.drop, DROP_ID, "Verified claims must bind the drop");
      assert.strictEqual(verified.of, 500, "Verified claims must bind the drop total");
    });

    it("should reject a receipt missing the drop object or the attestation", () => {
      const noDrop = receiptWithDrop("b".repeat(64));
      delete noDrop.drop;
      delete noDrop.editionAttestation;
      assert.throws(() => verifyDropEditionAttestation(noDrop));

      // Structurally invalid compact form.
      assert.throws(() => parseDropEditionAttestation("not.a.jws"));
    });
  });

  describe("Shop transactionService Pubky-app-specs", () => {
    const shopBody = () => ({
      schemaVersion: 1,
      recordType: "shop",
      ownerPubky: OTTO,
      revision: 1,
      createdAt: "2025-01-01T00:00:00Z",
      updatedAt: "2025-01-02T00:00:00Z",
      name: "Boots & Co",
      bio: "Quality hiking boots.",
      location: { countryCode: "US", region: "Oregon" },
      shippingPolicy: "Ships within 3 business days.",
      returnPolicy: "Returns accepted within 30 days.",
      vacationMode: false,
    });

    it("should accept a shop with an https transactionService", () => {
      const body = shopBody();
      body.transactionService = "https://tx.example.com";
      const { shop } = specsBuilder.createShop(body);
      assert.strictEqual(shop.toJson().transactionService, "https://tx.example.com");
    });

    it("should reject a non-https transactionService", () => {
      const body = shopBody();
      body.transactionService = "http://tx.example.com";
      assert.throws(() => specsBuilder.createShop(body));
    });

    it("should keep parsing a .6-shaped shop without the field", () => {
      const { shop } = specsBuilder.createShop(shopBody());
      const json = shop.toJson();
      assert.ok(
        !("transactionService" in json) || json.transactionService == null,
        "Absent transactionService must stay absent"
      );
    });
  });

  describe("Blob/File Pubky-app-specs", () => {
    it("should create blob with correct properties", () => {
      const length = 8
      const randomData = Array.from({length}, () => Math.floor(Math.random() * 256));
      const { blob, meta: blobMeta } = specsBuilder.createBlob(randomData);

      // Test meta properties
      assert.ok(blobMeta.id, "Blob should have an ID");
      assert.ok(blobMeta.url, "Blob should have a URL");
      const blobChunks = blobMeta.url.split("/")
      assert.strictEqual(blobChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(blobChunks[5], "blobs", "URL should contain blobs path");
      assert.strictEqual(blobChunks[6], blobMeta.id, "URL should contain blob ID");

      // Test blob content
      const blobJson = blob.toJson();
      // Blob JSON is just the raw array data
      assert.ok(Array.isArray(blobJson), "Blob should be an array");
      assert.strictEqual(blobJson.length, length, "Blob data should have correct length");

      // Create a file from the blob
      const { file, meta: fileMeta } = specsBuilder.createFile(
        "Pubky adventures", 
        blobMeta.url, 
        "application/pdf", 
        88
      );

      // Test meta properties
      assert.ok(fileMeta.id, "File should have an ID");
      assert.ok(fileMeta.url, "File should have a URL");
      const fileChunks = fileMeta.url.split("/")
      assert.strictEqual(fileChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(fileChunks[5], "files", "URL should contain files path");
      assert.strictEqual(fileChunks[6], fileMeta.id, "URL should contain file ID");

      // Test file content
      const fileJson = file.toJson();
      assert.strictEqual(fileJson.name, "Pubky adventures", "File name should match");
      assert.strictEqual(fileJson.src, blobMeta.url, "File src should reference blob URL");
      assert.strictEqual(fileJson.content_type, "application/pdf", "File content_type should match");
      assert.strictEqual(fileJson.size, 88, "File size should match");
      assert.ok(fileJson.created_at, "File should have created_at timestamp");
      assert.ok(typeof fileJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Feed Pubky-app-specs", () => {
    it("should create feed with correct properties", () => {
      const { feed, meta: feedMeta } = specsBuilder.createFeed(
        ["mountain","hike"], 
        "all", 
        "columns", 
        "recent", 
        "image", 
        "nature"
      );

      // Test meta properties
      assert.ok(feedMeta.id, "Feed should have an ID");
      assert.ok(feedMeta.url, "Feed should have a URL");
      assert.ok(feedMeta.url.includes(OTTO), "URL should contain user ID");
      assert.ok(feedMeta.url.includes("feeds"), "URL should contain feeds path");
      assert.ok(feedMeta.url.includes(feedMeta.id), "URL should contain feed ID");

      // Test feed content
      const feedJson = feed.toJson();
      assert.ok(feedJson.feed, "Feed should have feed property");
      assert.ok(Array.isArray(feedJson.feed.tags), "Feed tags should be an array");
      assert.deepStrictEqual(feedJson.feed.tags, ["mountain","hike"], "Feed tags should match");
      assert.strictEqual(feedJson.feed.reach, "all", "Feed reach should match");
      assert.strictEqual(feedJson.feed.layout, "columns", "Feed layout should match");
      assert.strictEqual(feedJson.feed.sort, "recent", "Feed sort should match");
      assert.strictEqual(feedJson.feed.content, "image", "Feed content should match");
      assert.strictEqual(feedJson.name, "nature", "Feed name should match");
      assert.ok(feedJson.created_at, "Feed should have created_at timestamp");
      assert.ok(typeof feedJson.created_at === "number", "created_at should be a number");
    });

    it("should create feed with wot reach and domain_tags", () => {
      const { feed } = specsBuilder.createFeed(
        ["rust"],
        "wot",
        "columns",
        "recent",
        "image",
        "WoT Feed",
        ["synonym"]
      );

      const feedJson = feed.toJson();
      assert.strictEqual(feedJson.feed.reach, "wot", "Feed reach should be wot");
      assert.deepStrictEqual(
        feedJson.feed.domain_tags,
        ["synonym"],
        "Feed domain_tags should match"
      );
    });

    it("should create feed with me reach without domain_tags", () => {
      const { feed } = specsBuilder.createFeed(
        null,
        "me",
        "list",
        "popularity",
        null,
        "My Posts"
      );

      const feedJson = feed.toJson();
      assert.strictEqual(feedJson.feed.reach, "me", "Feed reach should be me");
      assert.ok(
        feedJson.feed.domain_tags == null,
        "Feed domain_tags should be absent or null when not provided"
      );
    });
  });

  describe("Valid MIME Types", () => {
    it("should return an array of valid MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(Array.isArray(mimeTypes), "Should return an array");
      assert.ok(mimeTypes.length > 0, "Should have at least one MIME type");
    });

    it("should include common image MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("image/png"), "Should include image/png");
      assert.ok(mimeTypes.includes("image/jpeg"), "Should include image/jpeg");
      assert.ok(mimeTypes.includes("image/gif"), "Should include image/gif");
      assert.ok(mimeTypes.includes("image/webp"), "Should include image/webp");
    });

    it("should include common video MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("video/mp4"), "Should include video/mp4");
      assert.ok(mimeTypes.includes("video/mpeg"), "Should include video/mpeg");
    });

    it("should include common document MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("application/pdf"), "Should include application/pdf");
      assert.ok(mimeTypes.includes("application/json"), "Should include application/json");
      assert.ok(mimeTypes.includes("text/plain"), "Should include text/plain");
    });

    it("should be usable for file validation before upload", () => {
      const mimeTypes = getValidMimeTypes();
      
      // Valid file types
      assert.ok(mimeTypes.includes("image/png"), "image/png should be valid");
      assert.ok(mimeTypes.includes("application/pdf"), "application/pdf should be valid");
      
      // Invalid file types (not in the list)
      assert.ok(!mimeTypes.includes("application/x-executable"), "application/x-executable should not be valid");
      assert.ok(!mimeTypes.includes("application/x-msdownload"), "application/x-msdownload should not be valid");
    });

    it("should create file with valid MIME type from the list", () => {
      const mimeTypes = getValidMimeTypes();
      const validMimeType = mimeTypes[0]; // Pick the first valid MIME type
      
      const { blob, meta: blobMeta } = specsBuilder.createBlob([1, 2, 3, 4]);
      const { file } = specsBuilder.createFile(
        "test-file",
        blobMeta.url,
        validMimeType,
        100
      );
      
      const fileJson = file.toJson();
      assert.strictEqual(fileJson.content_type, validMimeType, "File should have valid MIME type");
    });
  });

  describe("Validation limits exports", () => {
    it("should expose validationLimits from JS exports", () => {
      assert.ok(validationLimits, "validationLimits should be defined");
      assert.deepStrictEqual(
        validationLimits,
        validationLimitsJson,
        "validationLimits should match validationLimits.json"
      );
      assert.strictEqual(
        validationLimits.userNameMinLength,
        3,
        "userNameMinLength should match the Rust limits"
      );
      assert.ok(
        Array.isArray(validationLimits.tagInvalidChars),
        "tagInvalidChars should be an array"
      );
    });

    it("getValidationLimits should return a copy that matches validationLimits", () => {
      const limitsCopy = getValidationLimits();

      assert.deepStrictEqual(
        limitsCopy,
        validationLimits,
        "getValidationLimits should match validationLimits"
      );
      assert.notStrictEqual(
        limitsCopy,
        validationLimits,
        "getValidationLimits should return a new object"
      );
    });

    it("builder.validationLimits should match the JS exports", () => {
      const builderLimits = JSON.parse(JSON.stringify(specsBuilder.validationLimits));

      assert.deepStrictEqual(
        builderLimits,
        validationLimits,
        "builder.validationLimits should match exported validationLimits"
      );
    });
  });
});
