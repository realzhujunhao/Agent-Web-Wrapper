"use client";

import { useState } from 'react';
import {
    MainContainer,
    TypingIndicator,
    ChatContainer,
    MessageList,
    Message,
    MessageInput,
    ConversationHeader,
    Avatar,
    Sidebar,
    ExpansionPanel,
    ArrowButton
} from '@chatscope/chat-ui-kit-react';

export default function ChatComponent({ messageList }) {
    console.log(`debug: ChatComponent args, ${JSON.stringify(messageList)}`)
    const [showSidebar, setShowSidebar] = useState(false);
    const toggleSidebar = () => {
        setShowSidebar(prev => !prev);
    };
    const messageElements = messageList.map((item, index) => {
        return item.role === "User" ?
            <Message
                key={`user-${index}`}
                model={{
                    direction: 'outgoing',
                    message: item.content,
                    position: 'single',
                    sender: 'user',
                    sentTime: ''
                }}
            >
                <Avatar
                    name="user"
                    src="/client.png"
                />
            </Message>
            :
            <Message
                key={`agent-${index}`}
                model={{
                    direction: 'incoming',
                    message: item.content,
                    position: 'single',
                    sender: 'agent',
                    sentTime: ''
                }}
            >
                <Avatar
                    name="agent"
                    src="/agent.png"
                />
            </Message>;
    });

    return (
        <MainContainer
            style={{
                position: "absolute", top: 0, left: 0, width: "100vw", height: "100vh"
            }}

        >
            <ChatContainer>
                <ConversationHeader>
                    <Avatar
                        name="养寿生得道AI"
                        src="/agent.png"
                    />
                    <ConversationHeader.Content
                        info='人工(在线/不在线 TODO) 需要人工介入时输入"转人工"'
                        userName="养寿生得道AI"
                    />
                    <ConversationHeader.Actions>
                        <ArrowButton
                            direction={showSidebar ? "right" : "left"}
                            onClick={toggleSidebar}
                        />
                    </ConversationHeader.Actions>
                </ConversationHeader>
                <MessageList typingIndicator={<TypingIndicator content="(AI/人工 TODO)正在输入" />}>
                    {messageElements}
                </MessageList>
                <MessageInput
                    placeholder="请描述您的问题"
                    onAttachClick={function attach() {
                        alert("debug: attach");
                    }}
                    onSend={function send() {
                        alert("debug: send");
                    }}
                />
            </ChatContainer>
            {
                showSidebar
                &&
                <Sidebar position="right"
                    style={{
                        overflow: 'hidden'
                    }}>
                    <ExpansionPanel
                        open
                        title="店铺信息"
                    >
                        店铺信息放这里
                    </ExpansionPanel>
                </Sidebar>
            }
        </MainContainer>
    )
}
