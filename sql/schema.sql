create table merchant
(
    id     serial
        constraint merchant_pkey primary key,
    name   varchar not null,
    secret varchar not null
);

create table account
(
    id       serial
        constraint account_pkey primary key,
    name     varchar not null,
    active   boolean,
    currency varchar not null,
    merch_id integer
        constraint acc_merch_fkey references merchant (id)
);

create table customer
(
    id           serial
        constraint customer_pkey
            primary key,
    phone        varchar not null,
    email        varchar not null,
    active       boolean,
    first_name   varchar not null,
    last_name    varchar not null,
    birth_date   date    not null,
    address      varchar not null,
    address2     varchar,
    city         varchar not null,
    state_region varchar,
    country      varchar not null,
    postal_code  varchar not null,
    merch_id     integer
        constraint cust_merch_fkey references merchant (id)
);

create table card
(
    id      serial
        constraint card_pkey primary key,
    type    varchar                  not null,
    created timestamp with time zone not null,
    cust_id integer
        constraint card_cust_fkey references customer (id),
    acc_id  integer
        constraint card_acc_fkey references account (id)
);

create table transaction
(
    id       serial
        constraint transaction_pkey primary key,
    order_id varchar not null,
    type     varchar not null,
    status   varchar not null
);

create table transaction_item
(
    id          serial
        constraint transaction_item_pkey primary key,
    amount      integer                  not null,
    created     timestamp with time zone not null,
    trans_id    integer
        constraint trans_itm_trans_fkey references transaction (id),
    src_acc_id  integer
        constraint trans_itm_src_acc_fkey references account (id),
    dest_acc_id integer
        constraint trans_itm_dest_acc_fkey references account (id),
    card_id     integer
        constraint trans_card_fkey references card (id)
);

create table transaction_fee
(
    id     serial
        constraint transaction_fee_pkey primary key,
    rate   float   not null,
    type   varchar not null,
    acc_id integer
        constraint trans_fee_acc_fkey references account (id)
);