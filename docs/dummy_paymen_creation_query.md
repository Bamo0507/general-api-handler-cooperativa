
> [!NOTE]
> The key after the payment is just a number in sha256

## First Payment

```redis
SET users:31559267D6F9152658557B4303CFF96EE29837E25EEEA5971104D57D7BAFE9A5:payments:5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9:date_created:2025-07-04
```

```redis
SET users:31559267D6F9152658557B4303CFF96EE29837E25EEEA5971104D57D7BAFE9A5:payments:5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9:comprobante_bucket:sup/sup.pdf
```

```redis
SET users:31559267D6F9152658557B4303CFF96EE29837E25EEEA5971104D57D7BAFE9A5:payments:5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9:ticket_number:39129391
```

> [!NOTE]
> Not adding affiliates now just for cause it won't be shown
