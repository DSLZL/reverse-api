#!/bin/bash

# Test Reverse-API Server Endpoints

set +e

# Configuration
API_URL="http://localhost:6969"
# Load tokens
DEEPSEEK_TOKEN=$(cat .deepseek_token 2>/dev/null || echo "")
QWEN_TOKEN=$(cat .qwen_token 2>/dev/null || echo "")

echo "🚀 Testing Reverse-API Server Endpoints"
echo "========================================"
echo ""

PASSED=0
FAILED=0

test_endpoint() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected=$5
    
    echo "Testing: $name"
    
    if [ "$method" == "GET" ]; then
        response=$(curl -s "$API_URL$endpoint")
    elif [ "$method" == "POST" ]; then
        response=$(curl -s -X POST "$API_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    elif [ "$method" == "DELETE" ]; then
        response=$(curl -s -X DELETE "$API_URL$endpoint")
    fi
    
    if echo "$response" | grep -q "$expected"; then
        echo "  ✅ PASSED"
        ((PASSED++))
    else
        echo "  ❌ FAILED - Response: $(echo $response | head -c 100)"
        ((FAILED++))
    fi
}

# Test 1: Health Check
echo "=== Basic Endpoints ==="
test_endpoint "Health Check" "GET" "/health" "" "status"

# Test 2: Models
test_endpoint "List Models" "GET" "/v1/models" "" "data"


# Test 3: DeepSeek Config and Response
if [ -n "$DEEPSEEK_TOKEN" ]; then
    echo ""
    echo "=== DeepSeek Model ==="
    
    # Configure DeepSeek
    response=$(curl -s -X POST "$API_URL/v1/config/deepseek" \
        -H "Content-Type: application/json" \
        -d "{\"token\": \"$DEEPSEEK_TOKEN\"}")
    
    if echo "$response" | grep -q "success"; then
        echo "  ✅ DeepSeek Config PASSED"
        ((PASSED++))
        
        # Create DeepSeek thread
        DS_THREAD=$(curl -s -X POST "$API_URL/v1/threads" \
            -H "Content-Type: application/json" \
            -d '{"model": "deepseek-r1"}' | jq -r '.id // empty')
        
        if [ -n "$DS_THREAD" ]; then
            echo "  ✅ Created DeepSeek thread"
            ((PASSED++))
            
            # Add message
            curl -s -X POST "$API_URL/v1/threads/$DS_THREAD/messages" \
                -H "Content-Type: application/json" \
                -d '{"role": "user", "content": "test"}' > /dev/null
            
            # Get response
            response=$(curl -s -X POST "$API_URL/v1/responses" \
                -H "Content-Type: application/json" \
                -d "{\"thread_id\": \"$DS_THREAD\", \"model\": \"deepseek-r1\"}")
            
            if echo "$response" | grep -q "completed"; then
                echo "  ✅ DeepSeek Response PASSED"
                ((PASSED++))
            else
                echo "  ❌ DeepSeek Response FAILED"
                ((FAILED++))
            fi
        fi
    else
        echo "  ❌ DeepSeek Config FAILED"
        ((FAILED++))
    fi
else
    echo ""
    echo "=== DeepSeek Model ==="
    echo "  ⚠️  SKIPPED (no token)"
fi

# Test 4: Qwen Config and Response
if [ -n "$QWEN_TOKEN" ]; then
    echo ""
    echo "=== Qwen Model ==="
    
    # Configure Qwen
    response=$(curl -s -X POST "$API_URL/v1/config/qwen" \
        -H "Content-Type: application/json" \
        -d "{\"token\": \"$QWEN_TOKEN\"}")
    
    if echo "$response" | grep -q "success"; then
        echo "  ✅ Qwen Config PASSED"
        ((PASSED++))
        
        # Create Qwen thread
        Q_THREAD=$(curl -s -X POST "$API_URL/v1/threads" \
            -H "Content-Type: application/json" \
            -d '{"model": "qwen3-max"}' | jq -r '.id // empty')
        
        if [ -n "$Q_THREAD" ]; then
            echo "  ✅ Created Qwen thread"
            ((PASSED++))
            
            # Add message
            curl -s -X POST "$API_URL/v1/threads/$Q_THREAD/messages" \
                -H "Content-Type: application/json" \
                -d '{"role": "user", "content": "test"}' > /dev/null
            
            # Get response
            response=$(curl -s -X POST "$API_URL/v1/responses" \
                -H "Content-Type: application/json" \
                -d "{\"thread_id\": \"$Q_THREAD\", \"model\": \"qwen3-max\"}")
            
            if echo "$response" | grep -q "completed"; then
                echo "  ✅ Qwen Response PASSED"
                ((PASSED++))
            else
                echo "  ❌ Qwen Response FAILED"
                ((FAILED++))
            fi
        fi
        
        # Test Qwen File Upload
        if [ -f "test_image.jpg" ]; then
            echo ""
            echo "=== Qwen Multimodal ==="
            response=$(curl -s -X POST "$API_URL/v1/files/upload" \
                -F "file=@test_image.jpg")
            
            if echo "$response" | grep -q '"id"'; then
                echo "  ✅ File Upload PASSED"
                ((PASSED++))
            else
                echo "  ❌ File Upload FAILED"
                ((FAILED++))
            fi
        fi
    else
        echo "  ❌ Qwen Config FAILED"
        ((FAILED++))
    fi
else
    echo ""
    echo "=== Qwen Model ==="
    echo "  ⚠️  SKIPPED (no token)"
fi

# Summary
echo ""
echo "================================"
echo "Test Summary:"
echo "  ✅ Passed: $PASSED"
echo "  ❌ Failed: $FAILED"
echo "================================"

if [ $FAILED -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
