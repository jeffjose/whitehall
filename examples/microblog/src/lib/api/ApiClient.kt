package com.example.microblog.lib.api

import com.example.microblog.models.*

// Mock API client for demonstration
// In real app, this would use Retrofit/Ktor
object ApiClient {

    // Auth
    suspend fun login(username: String, password: String): Result<String> {
        // Mock implementation
        return if (username.isNotEmpty() && password.length >= 6) {
            Result.success("mock-token-${username}")
        } else {
            Result.failure(Exception("Invalid credentials"))
        }
    }

    suspend fun signup(username: String, email: String, password: String): Result<String> {
        // Mock implementation
        return if (username.length >= 3 && email.contains("@") && password.length >= 6) {
            Result.success("mock-token-${username}")
        } else {
            Result.failure(Exception("Invalid data"))
        }
    }

    // Users
    suspend fun getUser(userId: String): Result<User> {
        // Mock data
        return Result.success(
            User(
                id = userId,
                username = "user_$userId",
                displayName = "User ${userId.take(8)}",
                bio = "This is a sample bio for user $userId",
                avatarUrl = "https://i.pravatar.cc/150?u=$userId",
                followersCount = 1234,
                followingCount = 567,
                isFollowing = false
            )
        )
    }

    // Posts
    suspend fun getFeed(page: Int = 0): Result<List<Post>> {
        // Mock feed data
        return Result.success(
            List(10) { index ->
                val postIndex = page * 10 + index
                Post(
                    id = "post_$postIndex",
                    authorId = "user_${postIndex % 5}",
                    authorName = "User ${postIndex % 5}",
                    authorAvatar = "https://i.pravatar.cc/150?u=${postIndex % 5}",
                    content = "This is post #$postIndex with some interesting content about Android development and Kotlin!",
                    timestamp = System.currentTimeMillis() - (postIndex * 3600000),
                    likesCount = (10..1000).random(),
                    commentsCount = (0..50).random(),
                    isLiked = (postIndex % 3 == 0)
                )
            }
        )
    }

    suspend fun getPost(postId: String): Result<Post> {
        return Result.success(
            Post(
                id = postId,
                authorId = "user_1",
                authorName = "Sample User",
                authorAvatar = "https://i.pravatar.cc/150?u=1",
                content = "This is the full content of post $postId. It contains detailed information and might be quite long.",
                timestamp = System.currentTimeMillis() - 7200000,
                likesCount = 42,
                commentsCount = 5,
                isLiked = false
            )
        )
    }

    suspend fun createPost(content: String): Result<Post> {
        val newPost = Post(
            id = "post_new_${System.currentTimeMillis()}",
            authorId = "current_user",
            authorName = "Current User",
            authorAvatar = "https://i.pravatar.cc/150?u=current",
            content = content,
            timestamp = System.currentTimeMillis(),
            likesCount = 0,
            commentsCount = 0,
            isLiked = false
        )
        return Result.success(newPost)
    }

    suspend fun likePost(postId: String): Result<Unit> {
        return Result.success(Unit)
    }

    suspend fun unlikePost(postId: String): Result<Unit> {
        return Result.success(Unit)
    }

    // Comments
    suspend fun getComments(postId: String): Result<List<Comment>> {
        return Result.success(
            List(5) { index ->
                Comment(
                    id = "comment_${postId}_$index",
                    postId = postId,
                    authorId = "user_$index",
                    authorName = "Commenter $index",
                    authorAvatar = "https://i.pravatar.cc/150?u=comment_$index",
                    content = "This is comment #$index on this post",
                    timestamp = System.currentTimeMillis() - (index * 1800000)
                )
            }
        )
    }

    suspend fun postComment(postId: String, content: String): Result<Comment> {
        val newComment = Comment(
            id = "comment_new_${System.currentTimeMillis()}",
            postId = postId,
            authorId = "current_user",
            authorName = "Current User",
            authorAvatar = "https://i.pravatar.cc/150?u=current",
            content = content,
            timestamp = System.currentTimeMillis()
        )
        return Result.success(newComment)
    }
}
