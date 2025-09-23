
>[!NOTE]
>The key after the payment is just a number in sha256

## First Payment
```redis
JSON.SET users:6D1A4829F2AFEE0E3FA226991F08720ABDC94DB81D452A658D9662F03F8E9664:payments:5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9 $ '{"date_created": "2025-08-25", "comprobante_bucket": "bucket_1/boleta.pdf", "ticket_number": "491292932","account_number": "92929090129", "status": "ON_REVISION", "quantity": 4.00, "comments": "Me quitaron mucho dinero"}'
```



>[!NOTE]
>Not adding affiliates now just for cause it won't be shown
