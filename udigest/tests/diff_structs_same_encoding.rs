#[derive(udigest::Digestable)]
struct BlogPost {
    title: String,
    content: String,
}

#[derive(udigest::Digestable)]
struct Issue {
    title: String,
    content: String,
}

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.udigest.example.issue_with_tag.v1")]
struct IssueWithTag {
    title: String,
    content: String,
}

#[test]
fn hash_the_same() {
    let title = "Ambiguous Encoding Bug";
    let content = "Using ambiguous encoding enables malicious payload manipulation under the exact same hash.";

    let blog_post = BlogPost {
        title: title.to_owned(),
        content: content.to_owned(),
    };
    let issue = Issue {
        title: title.to_owned(),
        content: content.to_owned(),
    };

    // Hash is the same even though the structures are different as they have the same encoding
    let hash1 = udigest::hash::<sha2::Sha256>(&blog_post);
    let hash2 = udigest::hash::<sha2::Sha256>(&issue);
    assert_eq!(hex::encode(hash1), hex::encode(hash2));

    // Use a unique domain separation tag to avoid ambiguity
    let issue_with_tag = IssueWithTag {
        title: title.to_owned(),
        content: content.to_owned(),
    };
    let hash3 = udigest::hash::<sha2::Sha256>(&issue_with_tag);
    assert_ne!(hex::encode(hash1), hex::encode(hash3));
}
