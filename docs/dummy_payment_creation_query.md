
>[!NOTE]
>The key after the payment is just a number in sha256

## First Payment

```redis
JSON.SET users:6D1A4829F2AFEE0E3FA226991F08720ABDC94DB81D452A658D9662F03F8E9664:payments:for_bryans_tests $ '{"date_created":"2025-10-12","account_number":"4","total_amount":100.0,"name":"Pago Para Algo","comments":null,"comprobante_bucket":"","ticket_number":"6","status":"ON_REVISION","being_payed":[{"model_type":"LOAN","amount":50.0,"model_key":"5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"}]}'
```

>[!NOTE]
>Not adding affiliates now just for cause it won't be shown
