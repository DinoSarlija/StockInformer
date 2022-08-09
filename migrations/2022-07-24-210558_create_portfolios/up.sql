-- Your SQL goes here
CREATE TABLE portfolios (
  id VARCHAR(36) DEFAULT uuid_generate_v4() NOT NULL ,
  name VARCHAR(255) NOT NULL ,
  created_at DATE NOT NULL ,
  is_deleted BOOLEAN NOT NULL,
  user_id VARCHAR(36) NOT NULL,
  CONSTRAINT pk_portfolio_id PRIMARY KEY ( id ),
  CONSTRAINT fk_user_portfolio FOREIGN KEY (user_id) REFERENCES users(id)
);