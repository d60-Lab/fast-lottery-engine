# 高性能抽奖系统 v1 规格说明

## 项目概述
构建一个高性能实时抽奖系统demo，支持高并发场景，采用Go后端 + Vue前端 + Docker-compose部署。

## 第一期目标 (specs分支)
专注于业务正确性验证，不追求极致性能，确保功能完整可用。

## 核心功能规格

### 1. 抽奖场景
- **实时抽奖**：用户登录后可立即参与转盘抽奖
- **转盘界面**：传统转盘样式，支持动画效果
- **即时结果**：抽中后即时显示结果并更新库存

### 2. 用户系统
- **用户登录**：支持用户注册和登录
- **用户身份**：每个用户有唯一标识，用于防重复抽奖
- **抽奖记录**：记录用户参与历史

### 3. 管理后台
- **奖项设置**：可配置多个奖项（一等奖、二等奖等）
- **奖品管理**：设置每个奖项的奖品数量、启用/禁用状态
- **概率配置**：设置各奖项的中奖概率
- **库存管理**：实时查看奖品剩余数量，库存预警
- **活动管理**：设置活动时间、状态管理
- **全局中奖历史**：分页展示所有用户中奖记录

### 4. 业务规则
- **库存控制**：奖品抽完后不再中该奖项，严格防止超卖
- **概率计算**：按配置概率进行随机抽取
- **公平性**：确保随机算法公平，不可预测
- **防刷机制**：
  - 单用户抽奖频率限制（如1分钟1次）
  - 活动时间限制（开始/结束时间）
  - 用户总抽奖次数限制（可选）
- **活动状态**：支持未开始/进行中/已结束/暂停状态
- **奖品状态**：支持启用/禁用状态

## 技术架构

### 后端 (Rust)
- **框架**：Axum
- **数据库**：PostgreSQL（主数据库）
- **缓存**：Redis（库存缓存和会话管理）
- **认证**：JWT Token
- **压测支持**：内置性能监控指标

### 前端 (Vue)
- **框架**：Vue 3 + TypeScript
- **UI库**：Element Plus 或 Ant Design Vue
- **动画**：转盘动画效果
- **管理后台**：前后端分离的管理界面

### 部署
- **容器化**：Docker + Docker-compose
- **服务拆分**：前端、后端、PostgreSQL、Redis分离
- **环境配置**：开发/测试/生产环境配置

## 性能目标 (第一期)
- **并发用户**：100-500并发
- **响应时间**：< 500ms
- **准确率**：100%业务正确性
- **可用性**：99.9%

## 压测方案设计

### 压测目标
- **验证并发能力**：找出系统的实际并发上限
- **验证业务正确性**：高并发下无超卖、无重复中奖
- **性能基线**：建立v1版本性能基准数据

### 压测场景
1. **注册压测**：批量注册1000万测试账号
2. **登录压测**：测试登录接口并发能力
3. **抽奖压测**：模拟真实抽奖场景
4. **混合压测**：登录+抽奖混合场景

### 压测工具
- **Vegeta**：HTTP压测工具（Go编写，但可测试任何HTTP服务）
- **自定义脚本**：批量注册和用户登录脚本
- **数据验证**：压测后验证库存数据一致性

### 压测指标
- **QPS**：每秒查询数
- **响应时间**：P50、P95、P99延迟
- **错误率**：业务错误和系统错误
- **并发数**：同时在线用户数
- **资源使用**：CPU、内存、数据库连接数

### 压测步骤
1. **环境准备**：Docker-compose启动完整环境
2. **数据准备**：批量注册测试用户，配置奖品数据
3. **基准测试**：单机单接口压测，建立基线
4. **业务测试**：完整抽奖流程压测
5. **数据验证**：检查库存一致性、中奖记录完整性
6. **报告输出**：生成压测报告和性能分析

## 数据模型设计

### 用户表 (users)
```sql
id, username, password, email, last_lottery_at, created_at, updated_at
```

### 活动表 (activities)
```sql
id, name, description, start_time, end_time, status, created_at, updated_at
```

### 奖项表 (prizes)
```sql
id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at
```

### 抽奖记录表 (lottery_records)
```sql
id, user_id, prize_id, prize_name, created_at
```

## API接口规划

### 用户相关
- POST /api/auth/login - 用户登录
- POST /api/auth/register - 用户注册
- GET /api/user/profile - 获取用户信息
- GET /api/user/lottery-history - 获取用户抽奖历史

### 抽奖相关
- POST /api/lottery/draw - 执行抽奖
- GET /api/lottery/prizes - 获取可抽奖品列表
- GET /api/lottery/result/{id} - 获取抽奖结果
- GET /api/lottery/global-history - 获取全局中奖历史（分页）

### 管理后台
- GET /admin/api/prizes - 获取奖品列表
- POST /admin/api/prizes - 创建奖品
- PUT /admin/api/prizes/{id} - 更新奖品
- DELETE /admin/api/prizes/{id} - 删除奖品
- GET /admin/api/statistics - 获取统计数据
- GET /admin/api/activities - 获取活动列表
- POST /admin/api/activities - 创建活动
- PUT /admin/api/activities/{id} - 更新活动状态

## 演进计划
- **specs分支**：规格设计和基础框架
- **v1分支**：基础功能实现，业务正确性
- **v2分支**：性能优化，高并发支持
- **v3分支**：完整功能，生产就绪

### 前端交互规范
- **抽奖动画**：转盘旋转3-5秒，期间禁用抽奖按钮
- **结果展示**：中奖结果展示2-3秒后自动关闭
- **错误处理**：库存不足、活动结束等需要友好提示
- **实时更新**：全局中奖历史直接查询，无缓存优化

## 压测工具设计

### 批量注册脚本
```bash
# 批量注册1000万用户
./scripts/batch-register.sh --count=10000000 --concurrent=100
```

### 压测执行脚本
```bash
# 登录压测
echo "POST http://localhost:8080/api/auth/login" | vegeta attack -duration=30s -rate=1000 | vegeta report

# 抽奖压测（需要携带token）
echo "POST http://localhost:8080/api/lottery/draw" | vegeta attack -duration=60s -rate=500 | vegeta report
```

### 数据验证脚本
```bash
# 验证库存一致性
./scripts/validate-inventory.sh

# 验证无重复中奖
./scripts/validate-duplicate-wins.sh
```

## 防超卖方案

### 核心策略：概率前置过滤 + Redis原子操作 + 数据库最终一致性

#### 性能优化：概率前置过滤
- **80%请求**：直接返回"谢谢惠顾"，不进入核心逻辑
- **20%请求**：走完整的库存扣减和概率计算
- **性能提升**：从100,000 QPS提升到500,000 QPS

#### 实现流程：
1. **概率前置过滤**：80%请求直接返回未中奖
2. **Redis预减库存**：使用`DECRBY`原子操作确保库存不超卖
3. **概率计算**：基于Redis中的实时库存计算中奖概率
4. **中奖记录**：记录中奖信息到Redis
5. **异步同步**：后台任务同步Redis数据到PostgreSQL

#### 关键技术点：
- 概率前置过滤大幅减少后端压力
- Redis原子操作保证并发安全
- 库存不足时立即返回"未中奖"
- 异步处理保证最终一致性
- 监控Redis和数据库数据差异

#### 伪代码示例：
```rust
// 1. 概率前置过滤（80%直接返回谢谢惠顾）
if rand::random::<f64>() < 0.8 {
    return LotteryResult::NotWon;
}

// 2. Redis预减库存
let remaining = redis.decrby("prize:1:stock", 1);
if remaining < 0 {
    // 库存不足，直接返回未中奖
    return LotteryResult::NotWon;
}

// 3. 计算中奖概率
let probability = calculate_probability(prize_id);
if rand::random::<f64>() < probability {
    // 4. 记录中奖
    let record_id = save_to_redis(prize_id, user_id);
    // 5. 异步同步到数据库
    async_sync_to_db(record_id);
    return LotteryResult::Won(prize_id);
} else {
    // 补偿Redis库存
    redis.incrby("prize:1:stock", 1);
    return LotteryResult::NotWon;
}
```

## 前端转盘动画方案

### 方案对比

#### CSS动画方案
**优势**：
- **性能优秀**：GPU硬件加速，流畅度高
- **开发简单**：代码简洁，维护成本低
- **响应式友好**：自动适配不同屏幕尺寸
- **兼容性好**：主流浏览器完美支持

**劣势**：
- 复杂动画效果受限
- 精确控制旋转角度较复杂

#### Canvas方案
**优势**：
- **效果丰富**：可定制复杂动画和特效
- **精确控制**：逐帧动画，控制精细
- **图形能力强**：适合复杂图形绘制

**劣势**：
- **性能开销大**：CPU渲染，内存占用高
- **开发复杂**：代码复杂度高
- **适配困难**：响应式实现复杂

### 技术选型：CSS动画
**选择理由**：
- 抽奖转盘是简单的旋转动画，CSS完全胜任
- 性能要求高，CSS动画GPU加速效果更好
- 开发维护简单，适合快速迭代
- 移动端兼容性更好

### CSS动画实现要点
```css
.wheel {
  transition: transform 3s cubic-bezier(0.2, 0.8, 0.2, 1);
  transform: rotate(0deg);
}

.wheel.spinning {
  transform: rotate(1440deg); /* 4圈 + 目标角度 */
}
```

## 管理后台认证方案

### 方案：硬编码账号密码 + .env管理

#### 实现方式：
- 使用`.env`文件存储管理员账号密码
- 硬编码在配置中，不存储在数据库
- 简单的用户名密码认证
- JWT token返回给前端

#### .env配置示例：
```env
ADMIN_USERNAME=admin
ADMIN_PASSWORD=your_secure_password
JWT_SECRET=your_jwt_secret_key
```

#### 认证流程：
1. 管理员输入用户名密码
2. 后端验证与`.env`配置匹配
3. 生成JWT token返回
4. 前端存储token用于后续请求

## 概率算法实现方案

### 选择：权重法

#### 实现原理：
- 每个奖品分配一个整数权重
- 总权重 = 所有奖品权重之和
- 随机数范围：[1, 总权重]
- 根据随机数落在哪个奖品的权重区间确定中奖结果

#### 示例：
```
奖品A：权重10
奖品B：权重20  
奖品C：权重30
谢谢惠顾：权重40
总权重 = 10 + 20 + 30 + 40 = 100

随机数范围：[1, 100]
- [1,10]：奖品A
- [11,30]：奖品B
- [31,60]：奖品C
- [61,100]：谢谢惠顾
```

#### 优点：
- 实现简单，计算高效
- 适合奖品数量不多的场景
- 整数运算，无精度问题

#### Rust实现伪代码：
```rust
fn calculate_prize(prizes: &[Prize]) -> Option<u32> {
    let total_weight: u32 = prizes.iter().map(|p| p.weight).sum();
    let random_num = rand::thread_rng().gen_range(1..=total_weight);
    
    let mut current_weight = 0;
    for prize in prizes {
        current_weight += prize.weight;
        if random_num <= current_weight {
            return Some(prize.id);
        }
    }
    None
}
```

## 错误码和异常处理规范

### 错误码设计原则
- **统一格式**：HTTP状态码 + 业务错误码
- **分层设计**：系统级错误、业务逻辑错误、参数校验错误
- **可读性强**：错误码包含模块信息和错误类型

### 错误码结构
```
HTTP状态码: 400/500
业务错误码: 模块(2位) + 错误类型(2位) + 具体错误(2位)
示例: 01-01-01 (用户模块-认证错误-密码错误)
```

### 异常处理策略
- **前端友好**：返回结构化的错误信息
- **日志记录**：记录详细错误堆栈用于排查
- **安全考虑**：生产环境不暴露敏感信息
- **重试机制**：网络错误自动重试

### 响应格式
```json
{
  "code": 400,
  "error_code": "01-01-01",
  "message": "密码错误",
  "data": null
}
```

## PostgreSQL表引擎和索引设计

### 表引擎选择
- **默认使用InnoDB**：支持事务、行级锁
- **无需特殊引擎**：PostgreSQL默认表引擎已足够

### 核心表索引设计

#### 用户表 (users)
```sql
-- 主键索引
PRIMARY KEY (id)
-- 用户名唯一索引
UNIQUE INDEX idx_users_username (username)
-- 创建时间索引（用于查询活跃用户）
INDEX idx_users_created_at (created_at)
```

#### 抽奖记录表 (lottery_records)
```sql
-- 主键索引
PRIMARY KEY (id)
-- 用户ID索引（查询用户中奖历史）
INDEX idx_records_user_id (user_id)
-- 奖品ID索引（统计奖品中奖情况）
INDEX idx_records_prize_id (prize_id)
-- 创建时间索引（时间范围查询）
INDEX idx_records_created_at (created_at)
-- 复合索引（用户+时间）
INDEX idx_records_user_created (user_id, created_at)
```

#### 奖品表 (prizes)
```sql
-- 主键索引
PRIMARY KEY (id)
-- 活动ID索引
INDEX idx_prizes_activity_id (activity_id)
-- 库存索引（快速查询有库存的奖品）
INDEX idx_prizes_stock (stock)
```

### 索引优化策略
- **读多写少**：适当增加索引
- **复合索引**：优先考虑常用查询组合
- **覆盖索引**：减少回表查询
- **定期维护**：清理无效索引，重建碎片索引

## 性能监控和日志配置

### 连接池监控
- **启用连接池监控**：监控连接数、空闲连接、等待连接
- **指标收集**：最大连接数、活跃连接数、等待连接数
- **告警机制**：连接池满时告警

### 慢查询日志
- **启用慢查询日志**：记录执行时间超过阈值的SQL
- **阈值设置**：建议100ms，根据压测结果调整
- **日志分析**：定期分析慢查询，优化SQL和索引

### 压测指标监控
- **QPS监控**：实时监控系统吞吐量
- **响应时间**：P50、P95、P99延迟
- **错误率**：HTTP错误码统计
- **资源使用**：CPU、内存、网络I/O

## 活动状态管理

### 状态定义
- **未开始**：活动配置完成，等待开始时间
- **进行中**：活动正在进行，用户可以抽奖
- **已结束**：活动结束时间到达，停止抽奖
- **暂停**：管理员手动暂停活动

### 状态流转规则
- 未开始 → 进行中：到达开始时间自动切换
- 进行中 → 已结束：到达结束时间自动切换
- 进行中 → 暂停：管理员手动暂停
- 暂停 → 进行中：管理员手动恢复
- 暂停 → 已结束：到达结束时间自动结束

### 状态验证
- 抽奖前检查活动状态
- 只有"进行中"状态允许抽奖
- 状态变更时清理相关缓存