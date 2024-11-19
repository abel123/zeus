# zeus

chanlun based quant trading
python + rust 缠论量化

### reference

- 核心逻辑使用的是 https://github.com/waditu/czsc ，原项目主要是为了离线处理。为了方便实时展示，作了少量改动。后逐步用 rust 重写


# zen-rs
zen in rust（rust 版缠论)

功能：
+ 多级别联立
+ 实时数据展示
+ 事件通知（系统桌面通知及飞书消息）
+ 支持 tradingview 的丰富画线工具
+ 支持多标的联立信号（如招商银行与上证指数叠加）
+ 行情回放
+ low code 自定义 dashboard
  
demo 示例:
![image](https://github.com/abel123/zen-rs/assets/3805243/03cc2304-e5f9-4623-bfad-2de2262846ef)
![image](https://github.com/abel123/zen-rs/assets/3805243/0832af62-9be9-42b1-9a59-821dd14c026e)

