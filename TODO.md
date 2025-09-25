## TODO SPRINT 8


## PRIORITARIOS
- [BLOCKED] SCRUM-202: Implementar unit testing para creación de pagos. Escribir test que simule la creación de un pago y valide la persistencia en la tabla pagos.
- [BLOCKED] SCRUM-252: Implementar unit testing para query de creación de pagos e integridad. Validar que los pagos creados sean consistentes y que los datos no se dupliquen.
- [X] SCRUM-257: Implementar unit testing para query que retorna cuotas de prestamo por id de usuario. Validar que los datos coincidan con la tabla cuotas y los tipos sean correctos.
- [ ] SCRUM-253: Implementar unit testing para query de creación de multas e integridad. Validar que las multas se asignen correctamente y que los datos sean consistentes con la tabla cuotas y prestamo_detalles.
- [blocked] SCRUM-201: Implementar unit testing para query de pagos por socio. Escribir test que verifique la obtención de pagos por usuario y valide los campos contra el esquema real.
- [X] SCRUM-255: Implementar GraphQL query para retornar cuotas por prestamo de los usuarios por su id. Validar que la query filtre correctamente y que los datos sean consistentes.
- [] SCRUM-314: Crear graphql query para retornar cuotas de los usuarios, OJO, estas son las que pagan todos


## RESTO
- [X] SCRUM-204: Crear tests para GraphQL query que retorna historial de pagos de usuario individual. Validar que los cálculos de historial sean correctos y que los errores se manejen bien para usuarios inexistentes.
- [X] SCRUM-199: Implementar unit testing para query de todos los socios. Verificar que la lista de socios se obtenga correctamente y que los campos estén alineados con la tabla usuarios.
- [X] SCRUM-259: Implementar GraphQL query para obtener pagos de todos los socios. Validar que la query retorne todos los pagos y que los campos estén alineados con la tabla pagos.
- [ ] SCRUM-260: Implementar GraphQL query para cambiar el estado de un pago y ponerle comentario. Validar que el estado se actualice correctamente y que el comentario se persista.
- [ ] SCRUM-261: Implementar unit testing de obtención de pagos y sus revisiones. Validar que los pagos y sus revisiones se obtengan correctamente y que los datos sean consistentes.
- [DONE] SCRUM-262: Implementar unit testing para retorno de cuotas de usuarios por id. Validar que la query filtre correctamente y que los datos sean consistentes con la tabla cuotas.
- [IN_PROGRESS] SCRUM-268: Preparar apartado en Insomnia para pruebas de multas. Crear colección de pruebas para endpoints relacionados con multas y validar los casos de error.