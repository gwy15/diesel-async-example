-- Your SQL goes here
CREATE TABLE `user` (
    `id`    bigint unsigned     NOT NULL AUTO_INCREMENT,
    `name`  varchar(255)        NOT NULL,
    `email` varchar(255)        NOT NULL,
    `created_at` datetime(3) NOT NULL DEFAULT now(3),
    `updated_at` datetime(3) NOT NULL DEFAULT now(3) ON UPDATE now(3),
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

