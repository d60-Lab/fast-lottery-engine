use once_cell::sync::Lazy;
use redis::Script;

// KEYS[1] = cooldown key, ARGV[1] = ttl seconds
pub static LUA_COOLDOWN_ONLY: Lazy<Script> = Lazy::new(|| {
    Script::new(r#"
        local cd = KEYS[1]
        local ttl = tonumber(ARGV[1])
        if redis.call('EXISTS', cd) == 1 then
            return 0
        end
        redis.call('SET', cd, '1', 'EX', ttl, 'NX')
        return 1
    "#)
});

// KEYS[1] = cooldown key, KEYS[2] = stock key, KEYS[3] = sold-delta key, ARGV[1] = ttl seconds
// returns: 1 if cooldown set and stock decremented; 0 if in cooldown; -1 if no stock
pub static LUA_COOLDOWN_AND_DECR: Lazy<Script> = Lazy::new(|| {
    Script::new(r#"
        local cd = KEYS[1]
        local sk = KEYS[2]
        local sold = KEYS[3]
        local ttl = tonumber(ARGV[1])
        if redis.call('EXISTS', cd) == 1 then
            return 0
        end
        local stock = tonumber(redis.call('GET', sk) or '0')
        if stock <= 0 then
            -- still set cooldown to throttle retries
            redis.call('SET', cd, '1', 'EX', ttl, 'NX')
            return -1
        end
        redis.call('DECR', sk)
        redis.call('INCR', sold)
        redis.call('SET', cd, '1', 'EX', ttl, 'NX')
        return 1
    "#)
});
