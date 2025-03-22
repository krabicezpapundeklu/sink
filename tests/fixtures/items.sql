INSERT INTO item (id, event_id, submit_date) VALUES(1, 1, '2025-01-01');
INSERT INTO item_header (item_id, name, value) VALUES (1, 'header-1', X'76616C75652D31'); -- value-1
INSERT INTO item_header (item_id, name, value) VALUES (1, 'header-2', X'76616C75652D32'); -- value-2
INSERT INTO item_body (item_id, body) VALUES (1, X'787878626F64792D31787878'); -- xxxbody-1xxx

INSERT INTO item (id, system, type, submit_date) VALUES(2, 'system-1', 'type-2', '2025-01-02');
INSERT INTO item_header (item_id, name, value) VALUES (2, 'header-1', X'76616C75652D31'); -- value-1
INSERT INTO item_body (item_id, body) VALUES (2, X'787878626F64792D32787878'); -- xxxbody-2xxx

INSERT INTO item (id, system, type, submit_date) VALUES(3, 'system-2', 'type-1', '2025-01-03');
INSERT INTO item_body (item_id, body) VALUES (3, X'69643A35');  -- id:5

INSERT INTO item (id, type, entity_event_id, submit_date) VALUES(4, 'event_payload', 1, '2025-01-04');
INSERT INTO item_body (item_id, body) VALUES (4, X'000102030405060708090A0B0C0D0E0FF0F1F2F3F4F5F6F7F8F9');

INSERT INTO item (id, type, entity_event_id, submit_date) VALUES(5, 'event_payload', 2, '2025-01-05');
INSERT INTO item_body (item_id, body) VALUES (5, X'');
