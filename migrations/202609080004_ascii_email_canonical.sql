UPDATE users
SET email_canonical = TRANSLATE(
    BTRIM(email),
    'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
    'abcdefghijklmnopqrstuvwxyz'
);
