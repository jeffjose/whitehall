package com.example.microblog.models

data class Post(
    val id: String,
    val authorId: String,
    val authorName: String,
    val authorAvatar: String,
    val content: String,
    val timestamp: Long,
    val likesCount: Int = 0,
    val commentsCount: Int = 0,
    val isLiked: Boolean = false
)
