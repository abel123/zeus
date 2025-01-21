# zeus

chanlun based quant trading
python + rust 缠论量化

### reference

- 核心逻辑使用的是 https://github.com/waditu/czsc ，原项目主要是为了离线处理。为了方便实时展示，作了少量改动。用 rust 重写核心逻辑，pyo3给 python 使用


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
![image](https://github.com/user-attachments/assets/f444ff22-b2fd-441a-b625-e6a36241e556)

![0003-GOOGL](https://github.com/user-attachments/assets/f201e09b-75c2-4092-a085-eae4e87a6c8e)
