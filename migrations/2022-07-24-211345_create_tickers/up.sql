-- Your SQL goes here
CREATE  TABLE tickers ( 
	id VARCHAR(36) DEFAULT uuid_generate_v4() NOT NULL ,
	name VARCHAR(255) NOT NULL ,
    portfolio_id VARCHAR(36) NOT NULL,
	is_deleted BOOLEAN NOT NULL,
	CONSTRAINT pk_ticker_id PRIMARY KEY ( id ),
    CONSTRAINT fk_ticker FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
);