sui client call --module fish --function mint \
--package 0x12bd30b99714b79190cbbf7afb4348d3b2912570769a033bde255c383378890e \
--args 0xed3c9b63eab6f1d9e841a48fff12b523908a574159f8034322f66952e7a8e61b \
1000000000 \
0x540ba39b0328acd14e100a8af76b7880e336abe08f806ada5643085794bd8aab \
--gas-budget 5000000

sui client call --module treasury --function deposit \
--package 0x12bd30b99714b79190cbbf7afb4348d3b2912570769a033bde255c383378890e \
--args 0x83d193fae7d73eb00fed5afb9fc27b24aa4eb4508be28b86d6ebbde372fb131d \
0x9041b5ef903d48f4f63cbb4f89adaaedf37a9009fde31f1e3f04fe5471760ca5 \
"init" 0x6 \
--gas-budget 5000000

sui client call --module user --function create_new_user \
--package 0x12bd30b99714b79190cbbf7afb4348d3b2912570769a033bde255c383378890e \
--args 0x1bb0ad9e0d561755f4510122614bed92c73f3c6a9744a9dfbe22108603755bdc \
0x83d193fae7d73eb00fed5afb9fc27b24aa4eb4508be28b86d6ebbde372fb131d \
0x354dc12bc2ff6bcf2b536a4dc655208f96408d2bac0611850df7e38103a902db \
0x6 \
--gas-budget 50000000



curl -X POST http://localhost:3000/test/create_profile -H "Content-Type: application/json" \
 -d '{"passport_id": "0xfc22cb40f745a871ded24d5273a13e4360d3e0f758f61c37d542dea0b83380d2"}'

curl -X POST http://localhost:3000/test/get_profile -H "Content-Type: application/json" \
 -d '{"passport_id": "0xfc22cb40f745a871ded24d5273a13e4360d3e0f758f61c37d542dea0b83380d2"}'




 ```
query GetObjectContent {
  object(address: "0x99102ed2eab7f4cd0e0a9ab42ec337a23a32bae22ccc3e9be229ba40d02ec5d5") {
    address
    asMoveObject {
      contents {
        json
      }
    }  
  }
}
 ```

 value:
 ```
 {
  "data": {
    "object": {
      "address": "0x99102ed2eab7f4cd0e0a9ab42ec337a23a32bae22ccc3e9be229ba40d02ec5d5",
      "asMoveObject": {
        "contents": {
          "json": {
            "id": "0x99102ed2eab7f4cd0e0a9ab42ec337a23a32bae22ccc3e9be229ba40d02ec5d5",
            "profiles": {
              "id": "0xacd0025cb33461761b83612a9faed2a30518825f846f880bed23841eb442d35d",
              "size": "2"
            },
            "ongoing_matches": [],
            "match_count": "0",
            "lobby_count": "0"
          }
        }
      }
    }
  }
}
 ```




 ```gql
query GetTableContent{
  owner(address: "0xacd0025cb33461761b83612a9faed2a30518825f846f880bed23841eb442d35d") {
    address
    dynamicFields(first: 1,after: "IDcytEITHS/kujFyIaaUfE9HvsY+4jakClJht23mdrsELgWTCwAAAAA=") {
	    pageInfo{
        hasNextPage
        endCursor
      }
      nodes {
        name { json }
        value {
          ... on MoveValue {
            json
          }
          ... on MoveObject {
            contents {
              json
            }
          }
        }
      }
    }
  }
}
 ```

value:
```
{
  "data": {
    "owner": {
      "address": "0xacd0025cb33461761b83612a9faed2a30518825f846f880bed23841eb442d35d",
      "dynamicFields": {
        "pageInfo": {
          "hasNextPage": false,
          "endCursor": "IIfoONtBbtIemqWWDn1aJp0gsvKrUy54kEKxV735rLQRXiKSCwAAAAA="
        },
        "nodes": [
          {
            "name": {
              "json": "0xfc22cb40f745a871ded24d5273a13e4360d3e0f758f61c37d542dea0b83380d2"
            },
            "value": {
              "json": "0x75909a18ce957735153bf02096da4937794eb58cfc284b4d02d798cac9fc52c4"
            }
          },
          {
            "name": {
              "json": "0xac22cb40f745a871ded24d5273a13e4360d3e0f758f61c37d542dea0b83380d2"
            },
            "value": {
              "json": "0x29ea12bf24c1882a4a6f46033ceaa14c05b311efc0ccacc219ddbc6aeb941382"
            }
          }
        ]
      }
    }
  }
}
```



```gql
query GetManagerStore {
  object(address: "0x75909a18ce957735153bf02096da4937794eb58cfc284b4d02d798cac9fc52c4") {
    address
    asMoveObject {
      contents {
        json
      }
    }  
  }
}
```

value:
```
{
  "data": {
    "object": {
      "address": "0x75909a18ce957735153bf02096da4937794eb58cfc284b4d02d798cac9fc52c4",
      "asMoveObject": {
        "contents": {
          "json": {
            "id": "0x75909a18ce957735153bf02096da4937794eb58cfc284b4d02d798cac9fc52c4",
            "avatar": "data:image/svg+xml;base64,value",
            "rating": "1000",
            "played": "0",
            "won": "0",
            "lost": "0"
          }
        }
      }
    }
  }
}
```