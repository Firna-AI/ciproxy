//! Dashboard identities, verified memberships, and durable role and install intent.

diesel::table! {
    dashboard_users (id) {
        id -> Uuid,
        github_id -> Int8,
        login -> Text,
        avatar_url -> Text,
        credentials -> Bytea,
        checked_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_memberships (user_id, github_id) {
        user_id -> Uuid,
        github_id -> Int8,
        owner -> Bool,
    }
}

diesel::table! {
    dashboard_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        digest -> Bytea,
        csrf_digest -> Bytea,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_login_states (id) {
        id -> Uuid,
        digest -> Bytea,
        verifier -> Bytea,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_roles (workspace_id, github_user_id) {
        workspace_id -> Uuid,
        github_user_id -> Int8,
        role -> Nullable<Text>,
        requested_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_install_intents (requester_id, github_organization_id) {
        requester_id -> Int8,
        github_organization_id -> Int8,
        workspace_id -> Uuid,
    }
}

diesel::table! {
    dashboard_takeouts (organization_id) {
        organization_id -> Uuid,
        workspace_id -> Uuid,
        installation_id -> Int8,
        completed -> Bool,
    }
}

diesel::table! {
    dashboard_initial_admin_receipts (workspace_id) {
        workspace_id -> Uuid,
    }
}
