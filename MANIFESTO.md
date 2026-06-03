# Flux Cookbook Manifesto

Este documento descreve como o Flux deve ser quando estiver pronto.

Ele não é um backlog, não é uma lista de tarefas e não deve vender maturidade antes da prova. O backlog de refactor deve ser escrito depois, a partir deste manifesto.

O papel do `flux-cookbook` é servir como especificação executável da ergonomia e do contrato público do Flux.

## Definição

Flux deve ser um toolkit explícito de repositórios para Rust, com suporte progressivo a agregados.

Flux não deve ser vendido como ORM.

O objetivo é entregar uma API pequena, previsível e backend-neutral onde isso fizer sentido, sem esconder comportamento perigoso de banco de dados.

Flux deve ajudar a aplicação a persistir entidades e agregados com menos repetição, mas sem assumir controle invisível sobre carregamento, cascata, transação, concorrência ou modelagem.

## Princípios

1. O core deve ser pequeno.
2. O core não deve depender de drivers.
3. Adapters devem carregar a complexidade de cada banco.
4. Persistência de entidade e persistência de grafo devem ser APIs diferentes.
5. Bulk write deve ser cidadão de primeira classe.
6. Paginação deve ser o caminho normal de leitura.
7. Campos tipados devem ser preferidos a strings.
8. IDs devem continuar tipados por entidade.
9. Semânticas diferentes entre bancos devem ser documentadas, não escondidas.
10. Exemplo executável vale mais que promessa em README.

## O Que Flux É

Flux é:

- um conjunto de traits para repositórios;
- um modelo comum para identidade, filtros, paginação e erros;
- uma forma explícita de mapear entidades para adapters;
- uma forma explícita de persistir agregados;
- uma API para reduzir boilerplate sem esconder o banco;
- uma base para adapters como PostgreSQL, MongoDB e SQL Server.

Flux deve permitir que a aplicação escreva código assim:

```rust
let events = postgres.repository::<Event>();

let filter = GenericFilter::<Event>::new()
    .gte(Event::fields().created_at, now - Duration::hours(1))
    .gte(Event::fields().score, 10)
    .and_group(|query| {
        query
            .or(|query| query.eq(Event::fields().status, "open"))
            .or(|query| query.eq(Event::fields().status, "paid"))
    })
    .order_by(Event::fields().created_at, OrderDirection::Desc);

let page = events.find_page_with_filter(filter, PageRequest::first(50)).await?;
```

E também código assim:

```rust
let orders = postgres.aggregate_repository::<Order>();

let saved = orders
    .save_graph(&order, GraphSaveMode::ReplaceChildren)
    .await?;
```

Essas duas operações não são a mesma coisa. Essa diferença é parte central da ergonomia.

## O Que Flux Não É

Flux não deve ser:

- um ORM completo;
- um sistema de lazy loading;
- um query builder universal;
- uma abstração que finge que SQL e MongoDB têm a mesma semântica;
- uma API que transforma `insert(&entity)` em persistência recursiva de grafo;
- uma camada que esconde cascata destrutiva;
- uma camada que remove a necessidade de pensar em transação;
- uma ferramenta que incentiva leitura sem limite em produção.

Se uma operação pode apagar dados relacionados, ela deve estar visível no nome da API, no modo de salvamento ou na metadata do agregado.

## Arquitetura Final

A separação desejada é:

```text
flux
  identidade
  filtros
  paginação
  traits de repositório
  contratos de bulk
  contratos de agregado
  metadados backend-neutral

flux-derive
  derives de Entity
  derives de campos tipados
  derives de mapeamento SQL
  derives de mapeamento MongoDB
  derives de agregado

flux-postgres
  conexão e pool
  repository PostgreSQL
  renderização SQL
  bind de parâmetros
  transações
  persistência de grafo relacional

flux-mongodb
  conexão e database
  repository MongoDB
  renderização BSON
  sessões e transações quando disponíveis
  persistência de grafo compatível com MongoDB

flux-sqlserver
  adapter futuro
  semântica própria
```

A direção de dependência deve ser sempre:

```text
app -> flux
app -> flux-derive
app -> flux-postgres
app -> flux-mongodb

flux-postgres -> flux
flux-mongodb  -> flux
flux-derive   -> flux
flux          -> nenhum driver de banco
```

O core não deve importar `tokio-postgres`, `mongodb`, `tiberius` ou qualquer driver equivalente.

## Contrato De Maturidade

O projeto deve declarar maturidade por área, não por marketing.

### Stable

Uma API só deve ser chamada de estável quando:

- possui exemplo executável no cookbook;
- possui teste automatizado no crate responsável;
- possui comportamento documentado para erro;
- possui semântica clara por adapter;
- não depende de comportamento implícito.

### Beta

Uma API pode ser beta quando:

- já existe implementação real;
- já possui exemplo executável;
- ainda pode sofrer ajuste de ergonomia;
- não deve ser vendida como contrato final.

### Experimental

Uma API deve ser experimental quando:

- envolve persistência de grafo;
- envolve cascade;
- envolve many-to-many;
- envolve ID gerado e propagado para filhos;
- envolve semântica transacional diferente entre adapters;
- ainda não tem testes suficientes de rollback, concorrência e perda acidental de dados.

### Proposed

Uma API deve ser proposta quando:

- existe apenas no documento;
- existe em exemplo conceitual;
- depende de refactor ainda não feito;
- depende de decisão de design ainda aberta.

README e `usage.md` devem seguir essa matriz. Se a API ainda não provou maturidade, a documentação deve dizer isso diretamente.

## Core

O crate `flux` deve conter apenas contratos que não pertencem a um banco específico.

O core deve conter:

- `Entity`;
- `EntityId`;
- `Repository`;
- `ReadRepository`;
- `WriteRepository`;
- `BulkRepository`;
- `AggregateRepository`;
- `GenericFilter`;
- `Field`;
- `FilterValue`;
- `FilterOp`;
- `PageRequest`;
- `Page`;
- `GraphSaveMode`;
- metadados de agregado.

O core não deve conter:

- pool PostgreSQL;
- client MongoDB;
- row SQL;
- BSON document;
- SQL string rendering;
- tipo externo de driver como contrato obrigatório;
- comportamento transacional concreto.

## Modelo De ID

Flux não deve ter um único `TypeId` como contrato principal de entidade.

O contrato correto é:

```rust
pub trait Entity {
    type Id: EntityId;

    fn id(&self) -> &Self::Id;
}
```

Cada entidade deve manter seu ID real:

```rust
pub struct Order {
    pub order_id: Uuid,
}

pub struct Product {
    pub product_id: i64,
}

pub struct Customer {
    pub customer_id: MongoObjectId,
}
```

Isso evita perder tipo, evita comparar IDs incompatíveis e mantém o compilador protegendo o domínio.

Um ID dinâmico pode existir, mas apenas como suporte auxiliar:

```rust
pub enum DynamicId {
    Uuid(Uuid),
    I64(i64),
    String(String),
    MongoObjectId(String),
}
```

Esse tipo serve para logs, metadados, ferramentas administrativas e mensagens genéricas. Ele não deve substituir `Entity::Id` nos repositórios.

## Campos Tipados

Filtros por string são úteis para casos dinâmicos, mas frágeis para uso normal.

O caminho principal deve ser campo tipado gerado por derive:

```rust
let filter = GenericFilter::<Event>::new()
    .eq(Event::fields().status, "open")
    .gte(Event::fields().created_at, start)
    .order_by(Event::fields().created_at, OrderDirection::Desc);
```

O campo deve carregar:

- entidade dona;
- nome físico do campo;
- tipo do valor;
- compatibilidade com filtro;
- compatibilidade com ordenação.

Conceitualmente:

```rust
pub struct Field<Entity, Value> {
    pub name: &'static str,
    marker: PhantomData<fn(Entity) -> Value>,
}
```

O derive deve gerar algo equivalente a:

```rust
impl Event {
    pub fn fields() -> EventFields {
        EventFields
    }
}

pub struct EventFields;

impl EventFields {
    pub const status: Field<Event, String> = Field::new("status");
    pub const created_at: Field<Event, DateTime<Utc>> = Field::new("created_at");
    pub const score: Field<Event, i32> = Field::new("score");
}
```

A sintaxe `Event::status` é tecnicamente possível, mas não deve ser o padrão recomendado porque força associated constants minúsculas e briga com convenção Rust.

`Event::fields().status` é a ergonomia preferida.

Strings devem continuar existindo como escape hatch:

```rust
let filter = GenericFilter::<Event>::new()
    .eq_dynamic("status", "open");
```

O nome dinâmico deve ser explícito. Isso deixa claro que o compilador não está protegendo aquele campo.

## Filtros

`GenericFilter` deve ser uma AST backend-neutral.

Ele deve suportar:

- `eq`;
- `ne`;
- `gt`;
- `gte`;
- `lt`;
- `lte`;
- `in_list`;
- `like` quando suportado;
- `is_null`;
- `is_not_null`;
- agrupamento `AND`;
- agrupamento `OR`;
- ordenação;
- múltiplas ordenações.

O core deve representar intenção. O adapter deve renderizar execução.

PostgreSQL renderiza SQL e parâmetros bindados.

MongoDB renderiza BSON document.

SQL Server renderiza sua própria variação SQL.

Quando um operador não tiver semântica segura em determinado adapter, o adapter deve retornar erro explícito em vez de tentar simular comportamento incorreto.

## Context Builder

O uso direto de repositories com conexão manual deve continuar possível, mas não deve ser a ergonomia principal para aplicações.

O Flux deve oferecer contexts por adapter:

```rust
let postgres = PostgresContext::connect(&database_url).await?;
let events = postgres.repository::<Event>();
let orders = postgres.aggregate_repository::<Order>();
```

Também deve aceitar pool externo:

```rust
let postgres = PostgresContext::from_pool(pool);
```

MongoDB deve seguir a mesma ideia:

```rust
let mongo = MongoContext::connect(&mongodb_url, "app").await?;
let customers = mongo.repository::<Customer>();
let orders = mongo.aggregate_repository::<Order>();
```

O context encapsula:

- conexão;
- pool;
- configuração;
- factories de repository;
- política transacional quando aplicável;
- hooks futuros de observabilidade.

O context não deve esconder semântica perigosa. Ele só deve reduzir boilerplate de infraestrutura.

## Repositório De Entidade

Persistência simples deve continuar simples:

```rust
let saved = products.insert(&product).await?;
let found = products.find_by_id(&product_id).await?;
let saved = products.update(&product).await?;
let saved = products.save(&product).await?;
let deleted = products.delete(&product_id).await?;
```

Essas operações são de uma entidade, uma tabela ou uma collection.

Elas não devem atravessar relações automaticamente.

`insert(&entity)` não deve salvar filhos.

`save(&entity)` não deve sincronizar agregado.

Esse limite é obrigatório para evitar ORM escondido.

## Paginação

Leitura paginada deve ser o caminho normal:

```rust
let page = products
    .find_page(PageRequest::first(50))
    .await?;
```

`find_all` não deve estar no caminho feliz da API.

Se existir, deve ser nomeado como operação explicitamente perigosa:

```rust
let all = products.find_all_unbounded().await?;
```

O nome deve deixar claro que a operação pode carregar a tabela ou collection inteira em memória.

## Bulk Write

Bulk write não é otimização futura. É parte do contrato principal.

O Flux deve suportar:

```rust
products.insert_many(&products_batch).await?;
products.update_many(&products_batch).await?;
products.save_many(&products_batch).await?;
products.delete_many(&product_ids).await?;
```

Persistência de agregado deve usar bulk internamente sempre que possível.

Um `save_graph` que salva filhos em loop N+1 não deve ser considerado pronto.

Adapters podem fazer chunk interno quando o banco tiver limite de parâmetros, tamanho de documento ou tamanho de batch.

## Persistência De Agregado

Persistência de grafo deve ser explícita:

```rust
orders.save_graph(&order, GraphSaveMode::ReplaceChildren).await?;
```

O nome `save_graph` comunica que a operação atravessa relações.

`GraphSaveMode` comunica como filhos devem ser tratados.

Modos esperados:

```rust
GraphSaveMode::AppendChildren
GraphSaveMode::UpsertChildren
GraphSaveMode::ReplaceChildren
```

### AppendChildren

Salva o root e adiciona filhos enviados.

Não apaga filhos ausentes.

### UpsertChildren

Salva o root e faz upsert dos filhos enviados.

Não apaga filhos ausentes.

### ReplaceChildren

Salva o root, faz upsert dos filhos enviados e remove filhos existentes que não aparecem mais no agregado.

Esse modo é destrutivo e deve ser testado com rigor.

## Regras De Segurança Do Grafo

Operações de grafo devem seguir regras estritas:

- toda operação de grafo deve ser transacional quando o adapter suportar;
- falha em filho deve fazer rollback do root;
- cascade delete deve exigir metadata explícita;
- `ReplaceChildren` deve apagar apenas filhos pertencentes ao parent correto;
- many-to-many deve tratar join table separadamente;
- join table com campos extras deve ser modelada como entidade própria;
- ID gerado pelo banco deve ser propagado para filhos antes do bulk insert;
- comportamento não suportado deve retornar erro claro.

O adapter não deve tentar ser inteligente ao ponto de salvar um grafo parcialmente sem avisar.

Se não houver transação real disponível, a documentação e o erro devem ser explícitos.

## PostgreSQL

PostgreSQL deve ser o adapter de referência.

Ele deve provar:

- CRUD;
- paginação;
- filtros tipados;
- bulk insert;
- bulk update;
- bulk upsert;
- generated IDs;
- `has_one`;
- `has_many`;
- `many_to_many`;
- transação em `save_graph`;
- rollback;
- `ReplaceChildren` seguro;
- cascade delete explícito.

O comportamento esperado deve ser validado por exemplos do cookbook e testes reais.

Enquanto PostgreSQL não provar esse contrato, não faz sentido declarar a API de grafo como estável.

## MongoDB

MongoDB deve compartilhar o contrato do core somente onde a semântica for honesta.

O adapter MongoDB deve suportar:

- repository por collection;
- filtros renderizados para BSON;
- paginação;
- bulk writes;
- ObjectId via tipo adapter-owned;
- graph persistence quando o modelo escolhido permitir;
- transações quando a configuração MongoDB suportar.

MongoDB não deve fingir ser SQL.

Quando uma relação for melhor modelada como documento embutido, isso deve aparecer na metadata e nos exemplos.

Quando uma relação for melhor modelada como referência, isso também deve aparecer.

O cookbook deve mostrar essas diferenças em vez de esconder.

## SQL Server

SQL Server deve ser tratado como adapter futuro.

Ele não deve bloquear o desenho do core, mas também não deve forçar promessas antes de existir implementação suficiente.

O core deve ser genérico o bastante para receber SQL Server depois.

O README não deve vender SQL Server como pronto se ele ainda não provar:

- CRUD;
- filtros;
- paginação;
- bulk;
- transação;
- graph persistence quando aplicável.

## Relacionamentos

### Has One

`has_one` deve representar uma relação direta entre root e filho opcional ou obrigatório.

O campo de relação não deve ser tratado como coluna do root.

### Has Many

`has_many` deve representar coleção filha com foreign key apontando para o root.

`ReplaceChildren` deve deletar apenas filhos cujo foreign key pertence ao root salvo.

### Many To Many

`many_to_many` deve funcionar bem para join table simples.

Join table com campos extras deve ser entidade explícita.

Isso é mais verboso, mas evita esconder dados importantes dentro de metadata.

## Erros

Flux deve preferir erros explícitos a comportamento implícito.

Exemplos de erro aceitável:

- operador de filtro não suportado pelo adapter;
- transação exigida mas indisponível;
- relation metadata inconsistente;
- tentativa de `ReplaceChildren` sem chave estrangeira confiável;
- tentativa de salvar grafo com ID gerado sem propagação implementada;
- tipo de campo não bindável no banco alvo.

Erro claro é melhor que abstração falsa.

## Documentação

A documentação deve separar:

- implementado;
- beta;
- experimental;
- proposto.

Nenhum README deve dizer que algo está pronto apenas porque existe assinatura pública.

Para ser considerado pronto, precisa existir:

- exemplo executável;
- teste automatizado;
- descrição de erro;
- descrição de semântica por adapter.

`usage.md` deve ensinar o contrato desejado.

O `flux-cookbook` deve provar o contrato com código real.

## Papel Do Cookbook

O cookbook deve conter crates pequenos, isolados e executáveis.

Cada crate deve provar uma situação.

O cookbook não deve conter exemplos falsos com `println!` simulando comportamento.

Cada exemplo deve ser guiado por traits reais do Flux:

- `ReadRepository`;
- `WriteRepository`;
- `BulkRepository`;
- `AggregateRepository`;
- `GenericFilter`;
- `PageRequest`;
- metadata derivada.

Quando a API ainda não existir, o exemplo não deve fingir que existe.

Nesse caso, o exemplo deve compilar contra o contrato atual ou ficar fora do cookbook até o refactor correspondente.

## Critério De Pronto

Flux deve ser considerado pronto para uma primeira versão forte quando:

1. `flux` core estiver pequeno e sem driver.
2. `flux-postgres` provar CRUD, filtros, paginação e bulk.
3. Filtros tipados estiverem disponíveis via derive.
4. `PostgresContext` existir como ergonomia principal.
5. `Entity::Id` estiver consolidado como contrato de identidade.
6. `find_all` não for caminho normal de produção.
7. `save_graph` for explícito e transacional.
8. `ReplaceChildren` tiver testes contra deleção errada.
9. Generated IDs funcionarem com propagação para filhos.
10. Exemplos PostgreSQL cobrirem CRUD, filtros, bulk, IDs e graph.
11. MongoDB documentar claramente o que é igual e o que é diferente.
12. README e `usage.md` refletirem maturidade real, não intenção.

## Respostas Às Dores

### README otimista demais

A solução é declarar maturidade por API e adapter.

O README deve apontar para exemplos executáveis e evitar afirmar suporte completo sem teste e exemplo correspondente.

### Projeto largo demais

A solução é ter arquitetura larga, mas entrega estreita.

O core pode ser preparado para múltiplos adapters. A primeira prova forte deve ser PostgreSQL.

MongoDB deve avançar com semântica própria. SQL Server deve ficar como futuro até provar contrato.

### Risco de virar ORM escondido

A solução é manter operações explícitas.

`insert`, `update`, `save` e `delete` operam uma entidade.

`insert_graph`, `save_graph` e `delete_graph` operam agregados.

Não deve existir lazy loading implícito.

### Aggregate persistence perigosa

A solução é tratar graph como API avançada, explícita e inicialmente experimental.

Graph precisa de transação, bulk, rollback, cascade opt-in e testes para modos destrutivos.

### SQL e MongoDB têm modelos diferentes

A solução é compartilhar contratos de alto nível, não fingir equivalência total.

O core define intenção.

O adapter define execução e limites.

### Filtros por string são frágeis

A solução é gerar campos tipados por derive e manter strings apenas como escape hatch explícito.

### Conexão manual prejudica ergonomia

A solução é criar contexts por adapter.

Contexts encapsulam pool e factories de repository, sem esconder semântica de banco.

### Um TypeId único parece simples, mas enfraquece o domínio

A solução é manter `Entity::Id` tipado.

Um ID dinâmico pode existir apenas para suporte auxiliar.

### Bulk não pode ser detalhe

A solução é colocar bulk no contrato principal.

Graph persistence deve usar bulk internamente.

### Cookbook precisa ser honesto

A solução é tratar cada crate de exemplo como uma prova executável.

Exemplo que só imprime intenção não serve como evidência de API.

## Decisão Estratégica

Flux deve reduzir promessa pública e aumentar prova executável.

O projeto deve continuar com arquitetura preparada para múltiplos bancos, mas a maturidade deve ser conquistada adapter por adapter.

O caminho correto é:

1. estabilizar o core;
2. tornar PostgreSQL a referência;
3. provar filtros tipados, paginação e bulk;
4. provar graph persistence com segurança;
5. documentar diferenças de MongoDB;
6. só então ampliar promessa pública.

Essa é a forma de manter ambição sem vender uma abstração frágil.
