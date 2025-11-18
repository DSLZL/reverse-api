use super::state::AppState;
use axum::{extract::State, response::Html};

pub async fn api_docs(State(_state): State<AppState>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reverse-API 文档</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
            line-height: 1.6;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 40px 20px;
        }
        
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px 20px;
            border-radius: 12px;
            margin-bottom: 40px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }
        
        .header h1 {
            font-size: 36px;
            margin-bottom: 10px;
        }
        
        .header p {
            font-size: 16px;
            opacity: 0.95;
        }
        
        .section {
            background: white;
            padding: 30px;
            margin-bottom: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        
        .section h2 {
            color: #667eea;
            font-size: 28px;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 2px solid #667eea;
        }
        
        .section h3 {
            color: #333;
            font-size: 20px;
            margin-top: 25px;
            margin-bottom: 15px;
        }
        
        .endpoint {
            background: #f8f9ff;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 15px;
            border-left: 4px solid #667eea;
        }
        
        .method {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 4px;
            color: white;
            font-weight: bold;
            margin-right: 10px;
            font-size: 12px;
        }
        
        .get { background: #28a745; }
        .post { background: #007bff; }
        .put { background: #ffc107; }
        .delete { background: #dc3545; }
        
        .path {
            font-family: 'Courier New', monospace;
            background: #e9ecef;
            padding: 2px 8px;
            border-radius: 4px;
            font-weight: 600;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 15px 0;
        }
        
        th {
            background: #f8f9ff;
            padding: 12px;
            text-align: left;
            font-weight: 600;
            border-bottom: 2px solid #667eea;
        }
        
        td {
            padding: 10px 12px;
            border-bottom: 1px solid #e9ecef;
        }
        
        .code-block {
            background: #2d2d2d;
            color: #f8f8f2;
            padding: 15px;
            border-radius: 6px;
            overflow-x: auto;
            margin: 15px 0;
            font-family: 'Courier New', monospace;
            font-size: 13px;
            line-height: 1.5;
        }
        
        .note {
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 15px;
            margin: 15px 0;
            border-radius: 4px;
        }
        
        .success {
            background: #d4edda;
            border-left: 4px solid #28a745;
        }
        
        .error {
            background: #f8d7da;
            border-left: 4px solid #dc3545;
        }
        
        .toc {
            background: #f8f9ff;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 30px;
        }
        
        .toc ul {
            list-style: none;
        }
        
        .toc li {
            margin: 8px 0;
        }
        
        .toc a {
            color: #667eea;
            text-decoration: none;
            transition: 0.2s;
        }
        
        .toc a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Reverse-API 文档</h1>
            <p>统一的多模型 AI API 接口（DeepSeek、Qwen）</p>
        </div>
        
        <div class="toc">
            <h3>目录</h3>
            <ul>
                <li><a href="section-endpoints">API 端点</a></li>
                <li><a href="section-monitoring">监控和统计</a></li>
                <li><a href="section-examples">使用示例</a></li>
                <li><a href="section-errors">错误处理</a></li>
            </ul>
        </div>
        
        <div class="section" id="section-overview">
            <h2>概述</h2>
            <p>Reverse-API 提供统一的接口来访问多个 AI 模型，包括：</p>
            <ul style="margin-left: 20px; margin-top: 10px;">
                <li><strong>Qwen</strong>：阿里巴巴 Qwen 模型（支持多模态）</li>
            </ul>
            
            <h3>基础信息</h3>
            <table>
                <tr>
                    <th>项目</th>
                    <th>说明</th>
                </tr>
                <tr>
                    <td>基础 URL</td>
                    <td><code>http://localhost:6969</code></td>
                </tr>
                <tr>
                    <td>API 版本</td>
                    <td>v1</td>
                </tr>
                <tr>
                    <td>认证方式</td>
                    <td>Bearer Token (可选)</td>
                </tr>
            </table>
        </div>
        
        <div class="section" id="section-auth">
            <h2>身份验证</h2>
            <p>大多数 API 端点无需认证即可访问。如果需要限制访问，可以在请求头中添加授权令牌：</p>
            <div class="code-block">Authorization: Bearer your-token</div>
        </div>
        
        <div class="section" id="section-endpoints">
            <h2>API 端点</h2>
            
            <h3>线程管理</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/threads</span></div>
                <p>创建新的对话线程</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "model": "grok-3-auto",
  "messages": [
    {
      "role": "user",
      "content": "Hello!"
    }
  ],
  "metadata": {}
}</div>
                <h4>响应</h4>
                <div class="code-block">{
  "id": "thread-123",
  "object": "thread",
  "created_at": 1234567890,
  "metadata": null
}</div>
            </div>
            
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/v1/threads</span></div>
                <p>列出所有线程</p>
            </div>
            
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/v1/threads/{thread_id}</span></div>
                <p>获取特定线程详情</p>
            </div>
            
            <div class="endpoint">
                <div><span class="method delete">DELETE</span><span class="path">/v1/threads/{thread_id}</span></div>
                <p>删除线程</p>
            </div>
            
            <h3>消息管理</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/threads/{thread_id}/messages</span></div>
                <p>添加消息到线程</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "role": "user",
  "content": "Your message here"
}</div>
            </div>
            
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/v1/threads/{thread_id}/messages</span></div>
                <p>列出线程的消息</p>
            </div>
            
            <h3>响应生成</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/responses</span></div>
                <p>为线程生成响应</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "thread_id": "thread-123"
}</div>
            </div>
            
            <h3>多模态功能 (Qwen)</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/files/upload</span></div>
                <p>上传文件到Qwen(需先配置Qwen token)，支持图片、视频、音频、文档</p>
                <h4>请求</h4>
                <p>Content-Type: multipart/form-data</p>
                <div class="code-block">参数名: file
文件类型: 图片(jpg, png), 视频(mp4), 音频(mp3, wav), 文档(txt, pdf)</div>
                <h4>响应</h4>
                <div class="code-block">{
  "id": "file-id-uuid",
  "name": "test_image.jpg",
  "size": 102400,
  "file_class": "vision"
}</div>
                <div class="note">上传后可在 /v1/responses 中使用 file_ids 参数传递文件ID</div>
            </div>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/images/generate</span></div>
                <p>使用Qwen生成图片</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "prompt": "一只可爱的小猫",
  "size": "1:1",
  "model": "qwen3-max",
  "download": true,
  "thread_id": "optional-for-continuous"
}</div>
                <h4>响应</h4>
                <div class="code-block">{
  "image_url": "https://cdn.qwenlm.ai/...",
  "prompt": "一只可爱的小猫",
  "chat_id": "chat-id",
  "response_id": "response-id",
  "local_path": "./generated/generated_image_xxx.png"
}</div>
                <div class="note success">设置 download=true 会自动下载到 ./generated/ 目录</div>
            </div>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/videos/generate</span></div>
                <p>使用Qwen生成视频（需要1-3分钟）</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "prompt": "一只小猫在草地上玩耍",
  "size": "16:9",
  "model": "qwen3-max",
  "download": true
}</div>
                <h4>响应</h4>
                <div class="code-block">{
  "video_url": "https://cdn.qwenlm.ai/...",
  "prompt": "一只小猫在草地上玩耍",
  "chat_id": "chat-id",
  "response_id": "response-id",
  "local_path": "./generated/generated_video_xxx.mp4"
}</div>
                <div class="note">视频生成耗时较长，请耐心等待。支持的尺寸: 1:1, 16:9, 9:16</div>
            </div>
            
            <h3>高级功能 (Qwen)</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/responses</span></div>
                <p>创建响应时支持高级功能</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "thread_id": "thread-123",
  "model": "qwen3-max",
  "file_ids": ["file-id-1", "file-id-2"],
  "instructions": "search",
  "instructions": "thinking"
}</div>
                <div class="note">
                    <strong>instructions 参数:</strong>
                    <ul style="margin-left: 20px; margin-top: 5px;">
                        <li><strong>search</strong>: 启用联网搜索功能</li>
                        <li><strong>thinking</strong>: 启用深度思考模式</li>
                        <li><strong>file_ids</strong>: 传递已上传的文件ID列表进行多模态分析</li>
                    </ul>
                </div>
            </div>
            
            <h3>连续对话</h3>
            
            <div class="note success">
                <strong>自动上下文保持</strong><br>
                API 会自动保存对话上下文。在同一个 thread_id 中连续发送消息时：
                <ul style="margin-left: 20px; margin-top: 5px;">
                    <li><strong>DeepSeek</strong>: 自动保持 session_id 和 message_id</li>
                    <li><strong>Qwen</strong>: 自动保持 chat_id 和 parent_id</li>
                    <li>其他模型通过消息历史维护上下文</li>
                </ul>
            </div>
            
            <h3>配置端点</h3>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/config/deepseek</span></div>
                <p>配置DeepSeek Token</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "token": "your_deepseek_token"
}</div>
            </div>
            
            <div class="endpoint">
                <div><span class="method post">POST</span><span class="path">/v1/config/qwen</span></div>
                <p>配置Qwen Token(支持多模态)</p>
                <h4>请求体</h4>
                <div class="code-block">{
  "token": "your_qwen_token"
}</div>
                <h4>响应</h4>
                <div class="code-block">{
  "status": "success",
  "message": "Qwen token configured"
}</div>
            </div>
            
            <h3>模型信息</h3>
            
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/v1/models</span></div>
                <p>列出支持的所有模型</p>
            </div>
            
            <h3>健康检查</h3>
            
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/health</span></div>
                <p>服务器健康状态检查</p>
            </div>
        </div>
        
        <div class="section" id="section-monitoring">
            <h2>监控和统计</h2>
            
            <h3>仪表板</h3>
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/dashboard</span></div>
                <p>访问实时性能监控仪表板</p>
            </div>
            
            <h3>统计数据</h3>
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/dashboard/stats</span></div>
                <p>获取请求统计数据</p>
                <h4>响应</h4>
                <div class="code-block">{
  "total_requests": 150,
  "successful_requests": 145,
  "failed_requests": 5,
  "last_request_time": 1234567890,
  "average_response_time": 1250
}</div>
            </div>
            
            <h3>实时请求</h3>
            <div class="endpoint">
                <div><span class="method get">GET</span><span class="path">/dashboard/requests</span></div>
                <p>获取最近的实时请求记录</p>
                <h4>响应</h4>
                <div class="code-block">[
  {
    "id": "1234567890",
    "timestamp": 1234567890,
    "method": "POST",
    "path": "/v1/responses",
    "status": 200,
    "duration_ms": 1250,
    "user_agent": "Python/3.9"
  }
]</div>
            </div>
        </div>
        
        <div class="section" id="section-examples">
            <h2>使用示例</h2>
            
            <h3>Python</h3>
            <div class="code-block">import requests

# 创建线程
response = requests.post('http://localhost:6969/v1/threads', json={
    'model': 'grok-3-auto',
    'messages': [{'role': 'user', 'content': 'Hello!'}]
})
thread_id = response.json()['id']

# 生成响应
response = requests.post('http://localhost:6969/v1/responses', json={
    'thread_id': thread_id
})
print(response.json())</div>
            
            <h3>cURL</h3>
            <div class="code-block">
# 创建线程
curl -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{"model":"grok-3-auto","messages":[{"role":"user","content":"Hello!"}]}'

# 生成响应
curl -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d '{"thread_id":"thread-123"}'</div>
        </div>
        
        <div class="section" id="section-errors">
            <h2>错误处理</h2>
            <p>API 使用标准 HTTP 状态码和 JSON 错误响应：</p>
            <table>
                <tr>
                    <th>状态码</th>
                    <th>说明</th>
                </tr>
                <tr>
                    <td>200 OK</td>
                    <td>请求成功</td>
                </tr>
                <tr>
                    <td>400 Bad Request</td>
                    <td>请求参数错误</td>
                </tr>
                <tr>
                    <td>404 Not Found</td>
                    <td>资源不存在</td>
                </tr>
                <tr>
                    <td>500 Internal Server Error</td>
                    <td>服务器错误</td>
                </tr>
            </table>
            
            <h3>错误响应格式</h3>
            <div class="code-block">{
  "error": "Not found",
  "status": 404,
  "details": "Thread with ID 'xyz' not found"
}</div>
        </div>
    </div>
</body>
</html>"#;
    Html(html.to_string())
}
