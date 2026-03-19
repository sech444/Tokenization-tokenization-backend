# Hybrid Architecture

```mermaid
flowchart TB
    %% Define mobile-friendly color scheme with better contrast
    classDef userClass fill:#e3f2fd,stroke:#1565c0,stroke-width:2px,color:#0d47a1,font-size:11px
    classDef frontendClass fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px,color:#4a148c,font-size:11px
    classDef apiClass fill:#e8f5e8,stroke:#388e3c,stroke-width:2px,color:#1b5e20,font-size:11px
    classDef dbClass fill:#fff8e1,stroke:#f57c00,stroke-width:2px,color:#e65100,font-size:11px
    classDef chainClass fill:#fce4ec,stroke:#c2185b,stroke-width:2px,color:#880e4f,font-size:11px
    classDef contractClass fill:#e8eaf6,stroke:#3f51b5,stroke-width:2px,color:#1a237e,font-size:11px
    classDef extClass fill:#f1f8e9,stroke:#689f38,stroke-width:2px,color:#33691e,font-size:11px

    %% User Layer - Simplified
    subgraph User["👤 User"]
        W[🔐 Wallet<br/>MetaMask]
        B[🌐 Browser<br/>Web3]
    end

    %% Frontend Layer - Condensed
    subgraph Frontend["🎨 Frontend"]
        R[⚛️ React 18+<br/>TypeScript]
        Wag[🔗 wagmi v2<br/>Hooks]
        E[📡 ethers.js<br/>Signing]
        UI[🎭 UI Libs<br/>Responsive]
    end

    %% API Backend Layer - Streamlined
    subgraph API["🚀 Backend"]
        N[🟢 Node.js<br/>Runtime]
        Nest[🏗️ NestJS<br/>Framework]
        Auth[🛡️ Auth<br/>JWT/OAuth2]
        Cache[⚡ Redis<br/>Cache]
    end

    %% Database Layer - Compact
    subgraph DB["🗄️ Storage"]
        PG[🐘 PostgreSQL<br/>ACID]
        Mongo[🍃 MongoDB<br/>NoSQL]
        IPFS[🌐 IPFS<br/>Assets]
    end

    %% Blockchain Layer - Simplified
    subgraph Chain["⛓️ Polygon"]
        Amoy[🧪 Testnet<br/>Testing]
        Main[💎 Mainnet<br/>Production]
    end

    %% Smart Contracts - Organized vertically
    subgraph Contracts["📋 Contracts"]
        T1[🏭 AssetTokenizer<br/>ERC-721/1155]
        T2[🏗️ TokenFactory<br/>Templates]
        T3[🏪 Marketplace<br/>Trading]
        T4[📜 Compliance<br/>KYC/AML]
        T5[🎁 Rewards<br/>Staking]
    end

    %% External Services - Compact
    subgraph External["🌍 External"]
        Oracle[🔮 Oracles<br/>Pricing]
        KYC[🔍 KYC<br/>Identity]
        Pay[💳 Payments<br/>Fiat]
    end

    %% Simplified connections for mobile
    User --> Frontend
    Frontend --> API
    Frontend --> Contracts
    API --> DB
    API --> Contracts
    API --> Chain
    API --> External
    Chain --> Contracts
    Contracts -.-> API

    %% Apply mobile-friendly styling
    class User,W,B userClass
    class Frontend,R,Wag,E,UI frontendClass
    class API,N,Nest,Auth,Cache apiClass
    class DB,PG,Mongo,IPFS dbClass
    class Chain,Amoy,Main chainClass
    class Contracts,T1,T2,T3,T4,T5 contractClass
    class External,Oracle,KYC,Pay extClass