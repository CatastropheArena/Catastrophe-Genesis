sui client call --module fish --function mint \
--package 0x1647c0f611b06c9e58fbaa0035c6cafc8e4aeb036fdfed421617751e2bd54d52 \
--args 0xdc6b4c97b9847b4b5cc437ed6803a5dbc17c170d96d5c487cb6aeec50d18e2d5 \
1000000000 \
0x540ba39b0328acd14e100a8af76b7880e336abe08f806ada5643085794bd8aab \
--gas-budget 5000000

sui client call --module treasury --function deposit \
--package 0x1647c0f611b06c9e58fbaa0035c6cafc8e4aeb036fdfed421617751e2bd54d52 \
--args 0x80bad00207901ec6e5440a3e39ed4990fe4eb0fa6cb78a440d4e9b99ad840ef9 \
0x91dbfe848374a61d8ea5157b023993dd5bc00cdffaa50e33c40c0644c998ea6d \
"init" 0x6 \
--gas-budget 5000000



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