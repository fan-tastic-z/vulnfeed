-- Add migration script here
ALTER TABLE
    vuln_information
ADD
    COLUMN detail_link TEXT NOT NULL DEFAULT '';
