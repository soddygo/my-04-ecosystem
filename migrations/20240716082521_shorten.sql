-- Add migration script here
create table if not exists shorten (
  id bigserial primary key,
  url text not null,
  created_at timestamptz not null default now()

)