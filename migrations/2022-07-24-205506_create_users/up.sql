-- Your SQL goes here
CREATE TABLE users (
  id VARCHAR(36) DEFAULT uuid_generate_v4() NOT NULL ,
  email VARCHAR(255) NOT NULL ,
  "password" VARCHAR(255) NOT NULL ,
  is_deleted BOOLEAN NOT NULL,
  CONSTRAINT pk_users_id PRIMARY KEY ( id )
);