-- Create the accounts_billing table
CREATE TABLE IF NOT EXISTS anything.accounts_billing (
    account_id uuid PRIMARY KEY REFERENCES basejump.accounts(id),
    stripe_customer_id text,
    stripe_data jsonb,
    free_trial_days integer DEFAULT 7,
    free_trial_task_limit integer DEFAULT 1000,
    free_trial_started_at timestamp with time zone,
    free_trial_ends_at timestamp with time zone,
    free_trial_task_usage bigint DEFAULT 0,
    trial_ended boolean DEFAULT false,
    total_task_usage bigint DEFAULT 0,
    total_execution_time_ms bigint DEFAULT 0,
    paying_customer boolean DEFAULT false,
    customer_status text DEFAULT 'trial',
    keep_processing_workflows boolean DEFAULT true,

    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);

-- Create a function to insert a new record into accounts_billing when a new account is created
CREATE OR REPLACE FUNCTION anything.create_account_billing()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO anything.accounts_billing (
        account_id,
        free_trial_started_at,
        free_trial_ends_at
    )
    VALUES (
        NEW.id,
        NOW(),
        NOW() + INTERVAL '7 days'
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to call the function when a new account is created
CREATE TRIGGER create_account_billing_trigger
AFTER INSERT ON basejump.accounts
FOR EACH ROW
EXECUTE FUNCTION anything.create_account_billing();

-- Protect the timestamps by setting created_at and updated_at to be managed by a trigger
CREATE TRIGGER set_accounts_billing_timestamp
BEFORE UPDATE ON anything.accounts_billing
FOR EACH ROW
EXECUTE PROCEDURE basejump.trigger_set_timestamps();

-- Enable RLS on the table
ALTER TABLE anything.accounts_billing ENABLE ROW LEVEL SECURITY;

-------------
-- Users should be able to read records that are owned by an account they belong to
--------------
create policy "Account members can select" on anything.accounts_billing
    for select
    to authenticated
    using (
    (account_id IN ( SELECT basejump.get_accounts_with_role()))
    );