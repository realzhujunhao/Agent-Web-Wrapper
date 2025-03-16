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

export default function ChatComponent() {
    const [showSidebar, setShowSidebar] = useState(false);
    const toggleSidebar = () => {
        setShowSidebar(prev => !prev);
    };

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
                        src="https://chatscope.io/storybook/react/assets/emily-xzL8sDL2.svg"
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
                    <Message
                        model={{
                            direction: 'incoming',
                            message: 'Hello my friend',
                            position: 'single',
                            sender: 'junhao',
                            sentTime: '15 mins ago'
                        }}
                    >
                        <Avatar
                            name="agent"
                            src="https://chatscope.io/storybook/react/assets/emily-xzL8sDL2.svg"
                        />
                    </Message>
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
