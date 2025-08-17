-- Add migration script here
ALTER TABLE public.users
    ALTER COLUMN ip_last_login TYPE VARCHAR(45) USING ip_last_login::TEXT;