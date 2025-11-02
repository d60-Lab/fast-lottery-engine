-- Demo seed data for quick testing
-- Activity: ongoing within a valid time window

-- activity id: 11111111-1111-1111-1111-111111111111
INSERT INTO activities (id, name, description, start_time, end_time, status, created_at, updated_at)
VALUES (
  '11111111-1111-1111-1111-111111111111',
  '新年抽奖活动',
  'Demo 活动（用于功能验证）',
  now() - interval '1 day',
  now() + interval '7 days',
  'ongoing',
  now(), now()
)
ON CONFLICT (id) DO NOTHING;

-- prizes for the above activity
-- total weight = 5 + 10 + 15 = 30  => 未中奖权重 = 70
INSERT INTO prizes (id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at)
VALUES
  ('22222222-2222-2222-2222-222222222222','11111111-1111-1111-1111-111111111111','一等奖','iPhone 15',5,5,5,true,now(),now()),
  ('33333333-3333-3333-3333-333333333333','11111111-1111-1111-1111-111111111111','二等奖','iPad',10,10,10,true,now(),now()),
  ('44444444-4444-4444-4444-444444444444','11111111-1111-1111-1111-111111111111','三等奖','京东卡100元',50,50,15,true,now(),now())
ON CONFLICT (id) DO NOTHING;
