use diself::{Cache, CacheConfig};
use diself::model::User;
use serde_json::json;

fn sample_user(id: &str) -> User {
    serde_json::from_value(json!({
        "id": id,
        "username": format!("user_{id}"),
        "discriminator": "0001"
    }))
    .expect("valid user json")
}

#[test]
fn cache_set_current_user_populates_current_user_and_user_cache() {
    let cache = Cache::new();
    let user = sample_user("123");

    cache.set_current_user(user.clone());

    assert_eq!(cache.current_user().map(|u| u.id), Some("123".to_string()));
    assert_eq!(cache.user_count(), 1);
    assert_eq!(cache.user("123").map(|u| u.username), Some("user_123".to_string()));
}

#[test]
fn cache_respects_disabled_user_cache() {
    let cache = Cache::with_config(CacheConfig {
        cache_users: false,
        cache_channels: true,
        cache_guilds: true,
        cache_relationships: true,
    });

    cache.cache_user(sample_user("999"));

    assert_eq!(cache.user_count(), 0);
    assert!(cache.user("999").is_none());
}

#[test]
fn cache_initialize_reads_ready_user() {
    let cache = Cache::new();
    let ready_payload = json!({
        "user": {
            "id": "555",
            "username": "ready_user",
            "discriminator": "1234"
        },
        "users": [],
        "guilds": [],
        "relationships": []
    });

    cache.initialize(ready_payload);

    let current = cache.current_user().expect("current user should be set");
    assert_eq!(current.id, "555");
    assert_eq!(current.username, "ready_user");
}

#[test]
fn cache_initialize_reads_read_states() {
    let cache = Cache::new();
    let ready_payload = json!({
        "user": {
            "id": "555",
            "username": "ready_user",
            "discriminator": "1234"
        },
        "users": [],
        "guilds": [],
        "relationships": [],
        "read_state": {
            "entries": [
                {
                    "id": "chan_1",
                    "read_state_type": 0,
                    "last_acked_id": "msg_9",
                    "badge_count": 2
                }
            ]
        }
    });

    cache.initialize(ready_payload);

    let read_state = cache.read_state("chan_1").expect("read state should be set");
    assert_eq!(read_state.last_acked_id.as_deref(), Some("msg_9"));
    assert_eq!(read_state.badge_count, Some(2));
}

#[test]
fn cache_updates_user_from_partial_presence_event() {
    let cache = Cache::new();
    cache.cache_user(sample_user("42"));

    cache.update_from_dispatch(
        "PRESENCE_UPDATE",
        &json!({
            "user": {
                "id": "42",
                "global_name": "Updated Name"
            },
            "status": "online",
            "activities": [],
            "client_status": {
                "desktop": "online"
            }
        }),
    );

    let user = cache.user("42").expect("user should still exist");
    assert_eq!(user.global_name.as_deref(), Some("Updated Name"));
    assert_eq!(user.username, "user_42");
    let presence = user.presence.expect("presence should be set");
    assert_eq!(presence.status, "online");
    assert_eq!(
        presence
            .client_status
            .and_then(|platform| platform.desktop)
            .as_deref(),
        Some("online")
    );
}

#[test]
fn cache_updates_channel_lifecycle_from_dispatch() {
    let cache = Cache::new();

    cache.update_from_dispatch(
        "CHANNEL_CREATE",
        &json!({
            "id": "c1",
            "type": 0,
            "name": "general"
        }),
    );
    assert!(cache.channel("c1").is_some());

    cache.update_from_dispatch(
        "CHANNEL_UPDATE",
        &json!({
            "id": "c1",
            "type": 0,
            "name": "general-2"
        }),
    );
    assert_eq!(
        cache
            .channel("c1")
            .and_then(|channel| channel.name)
            .as_deref(),
        Some("general-2")
    );

    cache.update_from_dispatch("CHANNEL_DELETE", &json!({ "id": "c1" }));
    assert!(cache.channel("c1").is_none());
}

#[test]
fn cache_updates_guild_and_relationship_from_dispatch() {
    let cache = Cache::new();

    cache.update_from_dispatch(
        "GUILD_CREATE",
        &json!({
            "id": "g1",
            "name": "Guild One",
            "channels": [
                {
                    "id": "cg1",
                    "type": 0,
                    "name": "chat"
                }
            ]
        }),
    );
    assert!(cache.guild("g1").is_some());
    assert!(cache.channel("cg1").is_some());

    cache.update_from_dispatch(
        "RELATIONSHIP_ADD",
        &json!({
            "id": "u999",
            "type": 1
        }),
    );
    assert!(cache.relationship("u999").is_some());

    cache.update_from_dispatch("RELATIONSHIP_REMOVE", &json!({ "id": "u999" }));
    assert!(cache.relationship("u999").is_none());

    cache.update_from_dispatch("GUILD_DELETE", &json!({ "id": "g1" }));
    assert!(cache.guild("g1").is_none());
}

#[test]
fn cache_updates_presence_from_ready_supplemental_merged_presences() {
    let cache = Cache::new();
    cache.cache_user(sample_user("1153408586026856468"));
    cache.cache_user(sample_user("726837366815195156"));

    cache.update_from_dispatch(
        "READY_SUPPLEMENTAL",
        &json!({
            "merged_presences": {
                "friends": [
                    {
                        "user_id": "1153408586026856468",
                        "status": "dnd",
                        "client_status": { "desktop": "dnd" },
                        "activities": [{ "name": "Visual Studio Code", "type": 0 }]
                    }
                ],
                "guilds": [
                    [],
                    [
                        {
                            "user_id": "726837366815195156",
                            "status": "online",
                            "client_status": { "desktop": "online" },
                            "activities": [{ "name": "League of Legends", "type": 0 }]
                        }
                    ]
                ]
            }
        }),
    );

    let user_friend = cache
        .user("1153408586026856468")
        .expect("friend user should be cached");
    let friend_presence = user_friend.presence.expect("friend presence should be set");
    assert_eq!(friend_presence.status, "dnd");

    let user_guild = cache
        .user("726837366815195156")
        .expect("guild user should be cached");
    let guild_presence = user_guild.presence.expect("guild presence should be set");
    assert_eq!(guild_presence.status, "online");
}

#[test]
fn cache_updates_merged_members_from_ready_supplemental() {
    let cache = Cache::new();

    cache.update_from_dispatch(
        "READY_SUPPLEMENTAL",
        &json!({
            "guilds": [
                { "id": "g1" }
            ],
            "merged_presences": {
                "friends": [],
                "guilds": []
            },
            "merged_members": [
                [
                    {
                        "user_id": "u1",
                        "roles": ["r1"],
                        "pending": false,
                        "mute": false,
                        "deaf": false,
                        "flags": 0
                    }
                ]
            ]
        }),
    );

    let member = cache
        .guild_member("g1", "u1")
        .expect("merged member should be present");
    assert_eq!(member.roles, vec!["r1".to_string()]);
}

#[test]
fn cache_updates_channel_from_passive_update() {
    let cache = Cache::new();
    cache.update_from_dispatch(
        "CHANNEL_CREATE",
        &json!({
            "id": "c-passive",
            "type": 0,
            "name": "general"
        }),
    );

    cache.update_from_dispatch(
        "PASSIVE_UPDATE_V1",
        &json!({
            "guild_id": "g1",
            "channels": [
                {
                    "id": "c-passive",
                    "last_message_id": "m77",
                    "last_pin_timestamp": "2026-02-23T00:00:00+00:00"
                }
            ]
        }),
    );

    let updated = cache
        .channel("c-passive")
        .expect("channel should be cached after passive update");
    assert_eq!(updated.last_message_id.as_deref(), Some("m77"));
    assert_eq!(
        cache
            .passive_channel_state("c-passive")
            .and_then(|state| state.last_message_id)
            .as_deref(),
        Some("m77")
    );
}
