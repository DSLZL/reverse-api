use super::state::AppState;
use axum::{extract::State, response::Html};

pub async fn dashboard(State(state): State<AppState>) -> Html<String> {
    let stats = state.get_stats().await;
    let threads = state.list_threads().await;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Grok API 仪表板</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@3/dist/chart.min.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        
        .header {{
            color: white;
            margin-bottom: 30px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }}
        
        .header h1 {{
            font-size: 32px;
            margin-bottom: 10px;
        }}
        
        .header p {{
            font-size: 16px;
            opacity: 0.9;
        }}
        
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        
        .stat-card {{
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}
        
        .stat-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 15px 40px rgba(0,0,0,0.3);
        }}
        
        .stat-label {{
            color: #666;
            font-size: 14px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 10px;
        }}
        
        .stat-value {{
            font-size: 32px;
            font-weight: bold;
            color: #667eea;
        }}
        
        .stat-unit {{
            font-size: 12px;
            color: #999;
            margin-top: 5px;
        }}
        
        .section {{
            background: white;
            border-radius: 12px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
        }}
        
        .section-title {{
            font-size: 24px;
            color: #333;
            margin-bottom: 20px;
            padding-bottom: 15px;
            border-bottom: 2px solid #667eea;
        }}
        
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        
        thead {{
            background: #f5f5f5;
        }}
        
        th {{
            padding: 15px;
            text-align: left;
            font-weight: 600;
            color: #333;
            border-bottom: 2px solid #ddd;
        }}
        
        td {{
            padding: 12px 15px;
            border-bottom: 1px solid #eee;
            color: #666;
        }}
        
        tr:hover {{
            background: #f9f9f9;
        }}
        
        .status-badge {{
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 600;
        }}
        
        .status-success {{
            background: #d4edda;
            color: #155724;
        }}
        
        .status-error {{
            background: #f8d7da;
            color: #721c24;
        }}
        
        .empty-state {{
            text-align: center;
            padding: 40px 20px;
            color: #999;
        }}
        
        .chart-container {{
            position: relative;
            height: 300px;
            margin-top: 20px;
        }}
        
        .refresh-info {{
            text-align: right;
            font-size: 12px;
            color: #999;
            margin-top: 20px;
        }}
        
        .refresh-badge {{
            background: #667eea;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            margin-left: 5px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Grok API 仪表板</h1>
            <p>实时性能监控与统计</p>
        </div>
        
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">总请求数</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">成功请求</div>
                <div class="stat-value" style="color: #28a745;">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">失败请求</div>
                <div class="stat-value" style="color: #dc3545;">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">平均响应时间</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">ms</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">活跃线程</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">成功率</div>
                <div class="stat-value" style="color: #17a2b8;">{}</div>
                <div class="stat-unit">%</div>
            </div>
        </div>
        
        <div class="section">
            <h2 class="section-title">📊 请求统计图表</h2>
            <div class="chart-container">
                <canvas id="requestsChart"></canvas>
            </div>
            <div id="chartData" style="display: none;"></div>
        </div>
        
        <div class="section">
            <h2 class="section-title">📈 实时请求</h2>
            <div id="requests-container"></div>
            <div class="refresh-info">
                数据每 5 秒自动刷新一次
                <span class="refresh-badge">实时</span>
            </div>
        </div>
    </div>
    
    <script>
        let requestsChart = null;
        
        async function updateDashboard() {{
            try {{
                const statsResp = await fetch('/dashboard/stats');
                const stats = await statsResp.json();
                
                const requestsResp = await fetch('/dashboard/requests');
                const requests = await requestsResp.json();
                
                // 更新统计数据
                if (stats.total_requests > 0) {{
                    document.getElementById('requests-container').innerHTML = 
                        '<table><thead><tr><th>时间</th><th>方法</th><th>路径</th><th>状态</th><th>耗时</th><th>User Agent</th></tr></thead><tbody>' +
                        (requests && requests.slice(0, 10).map(r => {{
                            const time = new Date(r.timestamp * 1000).toLocaleTimeString();
                            const statusClass = r.status >= 200 && r.status < 300 ? 'status-success' : 'status-error';
                            return '<tr><td>' + time + '</td><td>' + r.method + '</td><td>' + r.path + '</td>' +
                                   '<td><span class="status-badge ' + statusClass + '">' + r.status + '</span></td>' +
                                   '<td>' + (r.duration_ms / 1000).toFixed(2) + 's</td>' +
                                   '<td>' + (r.user_agent || 'N/A') + '</td></tr>';
                        }}).join('') || '') +
                        '</tbody></table>';
                }}
                
                // 更新图表
                if (requests && requests.length > 0) {{
                    updateChart(requests);
                }}
            }} catch (error) {{
                console.error('Failed to fetch data:', error);
            }}
        }}
        
        function updateChart(requests) {{
            const ctx = document.getElementById('requestsChart');
            if (!ctx) return;
            
            const last20 = requests.slice(0, 20).reverse();
            const labels = last20.map(r => new Date(r.timestamp * 1000).toLocaleTimeString());
            const durations = last20.map(r => (r.duration_ms / 1000).toFixed(2));
            
            if (requestsChart) {{
                requestsChart.destroy();
            }}
            
            requestsChart = new Chart(ctx, {{
                type: 'line',
                data: {{
                    labels: labels,
                    datasets: [{{
                        label: '响应时间 (秒)',
                        data: durations,
                        borderColor: '#667eea',
                        backgroundColor: 'rgba(102, 126, 234, 0.1)',
                        tension: 0.3,
                        fill: true
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{
                            beginAtZero: true,
                            title: {{
                                display: true,
                                text: '响应时间 (秒)'
                            }}
                        }}
                    }}
                }}
            }});
        }}
        
        // 初始化
        updateDashboard();
        setInterval(updateDashboard, 5000);
    </script>
</body>
</html>"#,
        stats.total_requests,
        stats.successful_requests,
        stats.failed_requests,
        stats.average_response_time,
        threads.len(),
        if stats.total_requests > 0 {
            ((stats.successful_requests as f64 / stats.total_requests as f64) * 100.0) as u64
        } else {
            0
        }
    );

    Html(html)
}

pub async fn dashboard_stats(
    State(state): State<AppState>,
) -> axum::Json<super::stats::RequestStats> {
    axum::Json(state.get_stats().await)
}

pub async fn dashboard_requests(
    State(state): State<AppState>,
) -> axum::Json<Vec<super::stats::LiveRequest>> {
    axum::Json(state.get_live_requests().await)
}
