 -- users
 CREATE TABLE IF NOT EXISTS users (
   id UUID PRIMARY KEY,
   username TEXT NOT NULL UNIQUE,
   password_hash TEXT NOT NULL,
   email TEXT NULL,
   last_lottery_at TIMESTAMPTZ NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
   updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );
 CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

 -- activities
 -- define enum type for activity status for clarity and safety
 DO $$ BEGIN
   CREATE TYPE activity_status AS ENUM ('planned','ongoing','paused','ended');
 EXCEPTION
   WHEN duplicate_object THEN NULL;
 END $$;

 CREATE TABLE IF NOT EXISTS activities (
   id UUID PRIMARY KEY,
   name TEXT NOT NULL,
   description TEXT NULL,
   start_time TIMESTAMPTZ NOT NULL,
   end_time TIMESTAMPTZ NOT NULL,
   status activity_status NOT NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
   updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );

 -- prizes
 CREATE TABLE IF NOT EXISTS prizes (
   id UUID PRIMARY KEY,
   activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
   name TEXT NOT NULL,
   description TEXT NULL,
   total_count BIGINT NOT NULL,
   remaining_count BIGINT NOT NULL,
   probability INT NOT NULL,
   is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
   updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );
 CREATE INDEX IF NOT EXISTS idx_prizes_activity_id ON prizes(activity_id);
 CREATE INDEX IF NOT EXISTS idx_prizes_stock ON prizes(remaining_count);

 -- lottery_records
 CREATE TABLE IF NOT EXISTS lottery_records (
   id UUID PRIMARY KEY,
   user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
   prize_id UUID NULL REFERENCES prizes(id) ON DELETE SET NULL,
   prize_name TEXT NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );
 CREATE INDEX IF NOT EXISTS idx_records_user_id ON lottery_records(user_id);
 CREATE INDEX IF NOT EXISTS idx_records_prize_id ON lottery_records(prize_id);
 CREATE INDEX IF NOT EXISTS idx_records_created_at ON lottery_records(created_at);
 CREATE INDEX IF NOT EXISTS idx_records_user_created ON lottery_records(user_id, created_at);
