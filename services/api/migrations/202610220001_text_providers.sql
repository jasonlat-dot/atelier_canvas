-- S1: text providers/catalog stay separate from demonstration image/video models.
CREATE TABLE text_providers (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 name text NOT NULL CHECK(char_length(name) BETWEEN 1 AND 120),
 protocol text NOT NULL CHECK(protocol='chat_completions'),
 base_url text NOT NULL,
 enabled boolean NOT NULL DEFAULT true,
 require_api_key boolean NOT NULL DEFAULT true,
 encrypted_api_key bytea,
 revision bigint NOT NULL DEFAULT 1 CHECK(revision>0),
 last_test_status text CHECK(last_test_status IN ('ok','error')),
 last_test_code text,
 last_test_at timestamptz,
 created_at timestamptz NOT NULL DEFAULT now(),
 updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE text_models (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 provider_id uuid NOT NULL REFERENCES text_providers(id),
 model_id text NOT NULL CHECK(char_length(model_id) BETWEEN 1 AND 256),
 label text NOT NULL CHECK(char_length(label) BETWEEN 1 AND 120),
 enabled boolean NOT NULL DEFAULT false,
 context_window integer NOT NULL CHECK(context_window BETWEEN 1024 AND 4000000),
 max_output_tokens integer NOT NULL CHECK(max_output_tokens>0 AND max_output_tokens<context_window),
 revision bigint NOT NULL DEFAULT 1 CHECK(revision>0),
 updated_at timestamptz NOT NULL DEFAULT now(),
 UNIQUE(provider_id,model_id)
);
CREATE INDEX idx_text_models_provider ON text_models(provider_id);
CREATE TABLE provider_admin_events (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 actor_id uuid NOT NULL REFERENCES users(id),
 action text NOT NULL,
 resource_id uuid NOT NULL,
 details jsonb NOT NULL,
 created_at timestamptz NOT NULL DEFAULT now()
);